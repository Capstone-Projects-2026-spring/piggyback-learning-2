from datetime import datetime
import json
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

from fastapi import HTTPException

from app.settings import (
    DOWNLOADS_DIR,
    EXPERT_QUESTION_TYPE_LABELS,
    EXPERT_QUESTION_TYPE_VALUES,
)
from app.services.question_generation_service import (
    _maybe_parse_json,
    resolve_question_file_param,
)
from app.services.video_files import find_primary_video_file, list_question_json_files


SEGMENT_MATCH_TOLERANCE = 1e-3


def normalize_segment_value(value: Any) -> float:
    try:
        return round(float(value), 3)
    except (TypeError, ValueError):
        return 0.0


def _parse_rank_value(value: Any) -> Optional[int]:
    if value is None:
        return None
    if isinstance(value, bool):
        return int(value)
    if isinstance(value, (int, float)):
        try:
            return int(value)
        except Exception:
            return None
    try:
        text = str(value).strip()
    except Exception:
        return None
    if not text:
        return None
    try:
        return int(text)
    except ValueError:
        try:
            return int(float(text))
        except Exception:
            return None


def _build_llm_rank_lookup(
    video_dir: Path, video_id: str
) -> Tuple[Dict[int, Dict[str, Optional[int]]], Dict[Tuple[Any, Any], Dict[str, Optional[int]]]]:
    by_index: Dict[int, Dict[str, Optional[int]]] = {}
    by_range: Dict[Tuple[Any, Any], Dict[str, Optional[int]]] = {}
    json_path = video_dir / "questions" / f"{video_id}.json"
    if not json_path.exists():
        return by_index, by_range

    try:
        data = json.loads(json_path.read_text(encoding="utf-8"))
    except Exception:
        return by_index, by_range

    segments = data.get("segments")
    if not isinstance(segments, list):
        return by_index, by_range

    for idx, seg in enumerate(segments):
        if not isinstance(seg, dict):
            continue
        result = seg.get("result") or {}
        questions = result.get("questions") or {}
        q_map: Dict[str, Optional[int]] = {}
        for qtype, info in questions.items():
            if isinstance(info, dict):
                q_map[qtype] = _parse_rank_value(info.get("rank"))
        by_index[idx] = q_map
        start = seg.get("start")
        end = seg.get("end")
        if start is not None and end is not None:
            by_range[(start, end)] = q_map

    return by_index, by_range


def load_expert_annotations(question_file: Path, video_id: str) -> Dict[str, Any]:
    annotations_path = question_file.with_suffix(question_file.suffix + ".expert.json")
    payload: Dict[str, Any] = {
        "video_id": video_id,
        "question_file": question_file.name,
        "annotations": [],
    }
    if annotations_path.exists():
        try:
            loaded = json.loads(annotations_path.read_text(encoding="utf-8"))
            if isinstance(loaded, dict):
                payload.update(
                    {
                        "annotations": loaded.get("annotations", []),
                    }
                )
        except Exception:
            pass
    return {
        "path": annotations_path,
        "data": payload,
    }


def serialize_question_segments(question_data: Dict[str, Any]) -> List[Dict[str, Any]]:
    segments: List[Dict[str, Any]] = []
    for idx, seg in enumerate(question_data.get("segments", [])):
        start = int(seg.get("start", 0))
        end = int(seg.get("end", start))
        result_raw = seg.get("result")
        parsed = _maybe_parse_json(result_raw)
        if isinstance(parsed, (dict, list)):
            display_payload = json.dumps(parsed, indent=2, ensure_ascii=False)
            parsed_for_js = parsed
        else:
            display_payload = (
                result_raw
                if isinstance(result_raw, str)
                else json.dumps(result_raw, indent=2, ensure_ascii=False)
            )
            parsed_for_js = None
        segments.append(
            {
                "index": idx,
                "start": start,
                "end": end,
                "parsed": parsed_for_js,
                "display": display_payload,
            }
        )
    return segments


def _build_annotations_map(annotations: List[Dict[str, Any]]) -> Dict[str, Any]:
    annotations_map: Dict[str, Any] = {}
    for entry in annotations:
        if not isinstance(entry, dict):
            continue
        key = f"{entry.get('start')}-{entry.get('end')}"
        annotations_map[key] = entry
    return annotations_map


def build_expert_preview_data(
    file: Optional[str], video: Optional[str], mode: Optional[str]
) -> Dict[str, Any]:
    mode_value = mode or "review"
    question_files = list_question_json_files()
    selected_file_path: Optional[Path] = None
    selected_file_rel: Optional[str] = None
    selection_error: Optional[str] = None

    if mode_value != "create" and not file and video:
        for item in question_files:
            if item["video_id"] == video:
                file = item["rel_path"]
                break

    if file:
        candidate = resolve_question_file_param(file)
        if candidate and candidate.exists():
            selected_file_path = candidate
            selected_file_rel = candidate.relative_to(DOWNLOADS_DIR).as_posix()
        else:
            selection_error = "Selected question JSON could not be found."

    segments_info: List[Dict[str, Any]] = []
    segments_for_js: List[Dict[str, Any]] = []
    existing_annotations: List[Dict[str, Any]] = []
    existing_annotations_map: Dict[str, Any] = {}
    selected_json_pretty: Optional[str] = None
    video_url: Optional[str] = None
    annotation_rel_path: Optional[str] = None
    selected_video_id: Optional[str] = None
    selected_file_name: Optional[str] = None

    if selected_file_path:
        selected_file_name = selected_file_path.name
        selected_video_dir = selected_file_path.parent.parent
        selected_video_id = selected_video_dir.name
        try:
            raw_data = json.loads(selected_file_path.read_text(encoding="utf-8"))
        except Exception:
            raw_data = {}
        segments_info = serialize_question_segments(raw_data)
        for segment in segments_info:
            parsed = segment.get("parsed")
            best_question = None
            questions_payload = None
            if isinstance(parsed, dict):
                questions_payload = parsed.get("questions")
                best_question = parsed.get("best_question")
            segments_for_js.append(
                {
                    "index": segment["index"],
                    "start": segment["start"],
                    "end": segment["end"],
                    "questions": questions_payload,
                    "best_question": best_question,
                }
            )
        selected_json_pretty = json.dumps(raw_data, indent=2, ensure_ascii=False)

        video_candidate = find_primary_video_file(selected_video_dir)
        if video_candidate:
            video_url = (
                f"/downloads/{video_candidate.relative_to(DOWNLOADS_DIR).as_posix()}"
            )

        annotations_bundle = load_expert_annotations(selected_file_path, selected_video_id)
        annotations_data = annotations_bundle["data"]
        annotations_list = annotations_data.get("annotations", [])
        if isinstance(annotations_list, list):
            existing_annotations = [entry for entry in annotations_list if isinstance(entry, dict)]
            existing_annotations_map = _build_annotations_map(existing_annotations)
        try:
            annotation_rel_path = (
                annotations_bundle["path"].relative_to(DOWNLOADS_DIR).as_posix()
            )
        except ValueError:
            annotation_rel_path = None
    elif mode_value == "create" and video:
        selected_video_id = video
        video_dir = DOWNLOADS_DIR / video
        if video_dir.exists():
            video_candidate = find_primary_video_file(video_dir)
            if video_candidate:
                video_url = (
                    f"/downloads/{video_candidate.relative_to(DOWNLOADS_DIR).as_posix()}"
                )

            expert_questions_dir = video_dir / "expert_questions"
            expert_file = expert_questions_dir / f"expert_{video}.json"

            if expert_file.exists():
                try:
                    expert_data = json.loads(expert_file.read_text(encoding="utf-8"))
                    annotations_list = (
                        expert_data.get("annotations", [])
                        if isinstance(expert_data, dict)
                        else []
                    )
                    if isinstance(annotations_list, list):
                        existing_annotations = [
                            entry for entry in annotations_list if isinstance(entry, dict)
                        ]
                        existing_annotations_map = _build_annotations_map(existing_annotations)
                    try:
                        annotation_rel_path = expert_file.relative_to(
                            DOWNLOADS_DIR
                        ).as_posix()
                    except ValueError:
                        annotation_rel_path = None
                except Exception:
                    pass

    return {
        "question_files": question_files,
        "selected_file_rel": selected_file_rel,
        "selected_file_name": selected_file_name,
        "selected_video_id": selected_video_id,
        "video_url": video_url,
        "segments": segments_info,
        "segments_for_js": segments_for_js,
        "existing_annotations": existing_annotations,
        "existing_annotations_map": existing_annotations_map,
        "selected_json_pretty": selected_json_pretty,
        "annotations_rel_path": annotation_rel_path,
        "selection_error": selection_error,
        "question_file_url": (
            f"/downloads/{selected_file_rel}" if selected_file_rel else None
        ),
        "mode": mode_value,
    }


def save_expert_annotation_payload(payload: Dict[str, Any]) -> Dict[str, Any]:
    if not isinstance(payload, dict):
        raise HTTPException(status_code=400, detail="Invalid payload.")

    mode = payload.get("mode", "review")

    if mode == "create":
        video_id = payload.get("video_id")
        if not video_id:
            raise HTTPException(
                status_code=400, detail="Missing video_id for create mode."
            )

        video_dir = DOWNLOADS_DIR / video_id
        if not video_dir.exists():
            raise HTTPException(status_code=400, detail="Video directory not found.")

        expert_questions_dir = video_dir / "expert_questions"
        expert_questions_dir.mkdir(exist_ok=True)

        expert_file = expert_questions_dir / f"expert_{video_id}.json"

        if expert_file.exists():
            try:
                expert_data = json.loads(expert_file.read_text(encoding="utf-8"))
                if not isinstance(expert_data, dict):
                    expert_data = {}
            except Exception:
                expert_data = {}
        else:
            expert_data = {}

        expert_data.setdefault("video_id", video_id)
        expert_data.setdefault("mode", "create")
        annotations_list = expert_data.setdefault("annotations", [])
    else:
        question_file = resolve_question_file_param(payload.get("file"))
        if not question_file or not question_file.exists():
            raise HTTPException(status_code=400, detail="Invalid question file.")

        video_dir = question_file.parent.parent
        video_id = video_dir.name

        annotations_bundle = load_expert_annotations(question_file, video_id)
        expert_data = annotations_bundle["data"]
        expert_data["video_id"] = video_id
        expert_data["question_file"] = question_file.name
        annotations_list = expert_data.setdefault("annotations", [])
        expert_file = annotations_bundle["path"]

    try:
        start = int(payload.get("start"))
        end = int(payload.get("end"))
    except (TypeError, ValueError):
        raise HTTPException(status_code=400, detail="Invalid segment bounds.")

    skip_requested = bool(payload.get("skip"))
    segment_index = payload.get("segment_index")
    try:
        segment_index = int(segment_index) if segment_index is not None else None
    except (TypeError, ValueError):
        segment_index = None

    timestamp = datetime.utcnow().isoformat(timespec="seconds") + "Z"

    if skip_requested:
        entry: Dict[str, Any] = {
            "segment_index": segment_index,
            "start": start,
            "end": end,
            "question_type": "skip",
            "question_type_label": "Skipped",
            "question": "(skipped)",
            "answer": "",
            "skipped": True,
            "saved_at": timestamp,
            "mode": mode,
        }
    else:
        question = (payload.get("question") or "").strip()
        answer = (payload.get("answer") or "").strip()
        question_type_raw = (payload.get("question_type") or "").strip().lower()

        if not question or not answer:
            raise HTTPException(
                status_code=400, detail="Question and answer are required."
            )
        if question_type_raw not in EXPERT_QUESTION_TYPE_VALUES:
            raise HTTPException(status_code=400, detail="Invalid question type.")

        entry = {
            "segment_index": segment_index,
            "start": start,
            "end": end,
            "question_type": question_type_raw,
            "question_type_label": EXPERT_QUESTION_TYPE_LABELS.get(
                question_type_raw, question_type_raw.title()
            ),
            "question": question,
            "answer": answer,
            "skipped": False,
            "saved_at": timestamp,
            "mode": mode,
        }

        if mode == "review":
            best_question_payload = payload.get("best_question")
            if isinstance(best_question_payload, dict):
                best_question_question = (
                    best_question_payload.get("question") or ""
                ).strip()
                best_question_answer = (
                    best_question_payload.get("answer") or ""
                ).strip()
                approved_raw = best_question_payload.get("approved")

                if isinstance(approved_raw, bool):
                    approved_value = approved_raw
                elif isinstance(approved_raw, str):
                    approved_value = approved_raw.lower() in {
                        "true",
                        "1",
                        "yes",
                        "approved",
                    }
                else:
                    approved_value = None

                comment_text = (best_question_payload.get("comment") or "").strip()

                if approved_value is False and not comment_text:
                    raise HTTPException(
                        status_code=400,
                        detail="Provide a comment when disapproving the best question.",
                    )

                if any(
                    [
                        best_question_question,
                        best_question_answer,
                        approved_value is not None,
                        comment_text,
                    ]
                ):
                    if approved_value is None:
                        approved_value = True

                    entry["best_question"] = {
                        "question": best_question_question,
                        "answer": best_question_answer,
                        "approved": approved_value,
                        "comment": comment_text if not approved_value else "",
                    }

    updated = False
    for idx, existing in enumerate(list(annotations_list)):
        if (
            isinstance(existing, dict)
            and existing.get("start") == start
            and existing.get("end") == end
        ):
            if (
                not skip_requested
                and mode == "review"
                and entry.get("best_question") is None
                and existing.get("best_question") is not None
            ):
                entry["best_question"] = existing.get("best_question")
            annotations_list[idx] = entry
            updated = True
            break

    if not updated:
        annotations_list.append(entry)

    annotations_list.sort(key=lambda item: (item.get("start", 0), item.get("end", 0)))

    expert_file.parent.mkdir(parents=True, exist_ok=True)
    try:
        expert_file.write_text(
            json.dumps(expert_data, indent=2, ensure_ascii=False), encoding="utf-8"
        )
    except Exception as exc:
        raise HTTPException(status_code=500, detail=f"Failed to store annotation: {exc}")

    try:
        annotation_rel = expert_file.relative_to(DOWNLOADS_DIR).as_posix()
    except ValueError:
        annotation_rel = None

    return {
        "success": True,
        "annotation": entry,
        "annotations_file": annotation_rel,
        "updated": updated,
        "mode": mode,
    }


def get_expert_questions_payload(video_id: str) -> Tuple[Dict[str, Any], int]:
    video_dir = DOWNLOADS_DIR / video_id
    questions_dir = video_dir / "expert_questions"
    file_path = questions_dir / "expert_questions.json"

    if not video_dir.exists() or not questions_dir.exists() or not file_path.exists():
        return {"success": True, "video_id": video_id, "questions": []}, 200

    try:
        data = json.loads(file_path.read_text(encoding="utf-8"))
    except Exception as exc:
        return {
            "success": False,
            "message": f"Unable to read expert questions: {exc}",
            "questions": [],
        }, 500

    questions = data.get("questions") if isinstance(data, dict) else []
    if not isinstance(questions, list):
        questions = []

    return {"success": True, "video_id": video_id, "questions": questions}, 200


def save_expert_question_payload(payload: Dict[str, Any]) -> Tuple[Dict[str, Any], int]:
    video_id = str(payload.get("videoId") or payload.get("video_id") or "").strip()
    if not video_id:
        return {"success": False, "message": "videoId is required"}, 400

    video_dir = DOWNLOADS_DIR / video_id
    if not video_dir.exists():
        return {"success": False, "message": "Video not found"}, 404

    segment_start_value = normalize_segment_value(payload.get("segmentStart"))
    segment_end_value = normalize_segment_value(payload.get("segmentEnd"))
    timestamp_value = normalize_segment_value(
        payload.get("timestamp", segment_end_value)
    )

    skipped = bool(payload.get("skipped") or payload.get("skip") or payload.get("isSkipped"))
    skip_reason = str(payload.get("skipReason") or payload.get("skip_reason") or "").strip()

    if segment_end_value <= segment_start_value:
        segment_end_value = segment_start_value

    question_type = (
        str(payload.get("questionType") or payload.get("question_type") or "")
        .strip()
        .lower()
    )
    question_text = str(payload.get("question") or "").strip()
    answer_text = str(payload.get("answer") or "").strip()

    if skipped:
        question_type = ""
        question_text = ""
        answer_text = ""
    else:
        if question_type not in EXPERT_QUESTION_TYPE_VALUES:
            return {"success": False, "message": "Invalid question type"}, 400

        if not question_text or not answer_text:
            return {"success": False, "message": "Question and answer are required"}, 400

    questions_dir = video_dir / "expert_questions"
    questions_dir.mkdir(parents=True, exist_ok=True)
    file_path = questions_dir / "expert_questions.json"

    try:
        stored = (
            json.loads(file_path.read_text(encoding="utf-8")) if file_path.exists() else {}
        )
    except Exception:
        stored = {}

    if not isinstance(stored, dict):
        stored = {}

    questions_list = stored.get("questions")
    if not isinstance(questions_list, list):
        questions_list = []

    def matches_existing(entry: Dict[str, Any]) -> bool:
        existing_start = normalize_segment_value(entry.get("segment_start"))
        existing_end = normalize_segment_value(entry.get("segment_end"))
        return (
            abs(existing_start - segment_start_value) < SEGMENT_MATCH_TOLERANCE
            and abs(existing_end - segment_end_value) < SEGMENT_MATCH_TOLERANCE
        )

    questions_list = [q for q in questions_list if not matches_existing(q)]

    entry = {
        "segment_start": segment_start_value,
        "segment_end": segment_end_value,
        "timestamp": timestamp_value,
        "question_type": question_type if not skipped else None,
        "question": question_text,
        "answer": answer_text,
        "skipped": skipped,
        "skip_reason": skip_reason,
        "updated_at": datetime.utcnow().isoformat(),
    }

    questions_list.append(entry)
    questions_list.sort(key=lambda q: normalize_segment_value(q.get("segment_start")))

    stored["video_id"] = video_id
    stored["questions"] = questions_list

    try:
        file_path.write_text(json.dumps(stored, indent=2), encoding="utf-8")
    except Exception as exc:
        return {"success": False, "message": f"Failed to write expert questions: {exc}"}, 500

    message = "Segment marked as skipped." if skipped else "Expert question saved."
    return {
        "success": True,
        "message": message,
        "updatedAt": entry["updated_at"],
        "skipped": skipped,
    }, 200


def save_final_questions_payload(payload: Dict[str, Any]) -> Tuple[Dict[str, Any], int]:
    video_id = str(payload.get("videoId") or "").strip()
    if not video_id:
        return {"success": False, "message": "videoId is required"}, 400

    video_dir = DOWNLOADS_DIR / video_id
    if not video_dir.exists():
        return {"success": False, "message": "Video not found"}, 404

    final_data = payload.get("data")
    if not final_data:
        return {"success": False, "message": "No data provided"}, 400

    final_questions_dir = video_dir / "final_questions"
    final_questions_dir.mkdir(parents=True, exist_ok=True)
    final_file_path = final_questions_dir / "final_questions.json"

    try:
        final_data["saved_at"] = datetime.utcnow().isoformat()
        final_data["video_id"] = video_id

        segments = final_data.get("segments")
        if not isinstance(segments, list):
            segments = []
        final_data["segments"] = segments

        llm_by_index, llm_by_range = _build_llm_rank_lookup(video_dir, video_id)

        for idx, seg in enumerate(segments):
            if not isinstance(seg, dict):
                continue

            raw_index = seg.get("segmentIndex", idx)
            try:
                seg_index = int(raw_index)
            except (TypeError, ValueError):
                seg_index = idx

            llm_rankings = llm_by_index.get(seg_index)
            if llm_rankings is None:
                start = seg.get("start")
                end = seg.get("end")
                llm_rankings = llm_by_range.get((start, end))
            if llm_rankings is None:
                llm_rankings = {}

            ai_questions = seg.get("aiQuestions")
            if not isinstance(ai_questions, list):
                seg["aiQuestions"] = []
                continue

            for question in ai_questions:
                if not isinstance(question, dict):
                    continue

                raw_expert = question.get("expert_ranking")
                if raw_expert is None:
                    raw_expert = question.get("ranking")
                expert_rank = _parse_rank_value(raw_expert)
                if expert_rank is None and question.get("trashed"):
                    expert_rank = 0
                question["expert_ranking"] = expert_rank
                if "ranking" in question:
                    del question["ranking"]

                llm_rank = None
                q_type = question.get("type")
                if q_type and isinstance(llm_rankings, dict):
                    llm_rank = llm_rankings.get(q_type)
                if llm_rank is None:
                    llm_rank = _parse_rank_value(question.get("llm_ranking"))
                question["llm_ranking"] = llm_rank

        final_file_path.write_text(json.dumps(final_data, indent=2), encoding="utf-8")

        return {
            "success": True,
            "message": "Final questions saved successfully",
            "filepath": f"downloads/{video_id}/final_questions/final_questions.json",
            "saved_at": final_data["saved_at"],
        }, 200

    except Exception as exc:
        return {"success": False, "message": f"Failed to save final questions: {exc}"}, 500
