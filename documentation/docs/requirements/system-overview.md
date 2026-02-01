---
sidebar_position: 1
---

# System Overview

## Project Abstract

This document proposes improvements to an existing application, Piggyback Learning. This application aims to help children retain focus while watching educational videos by automatically generating quizzes based on the video they’re watching. As the video plays, it will occasionally pause while a short quiz about the events of the video is queued.

## Conceptual Design

Piggyback Learning is a FastAPI-based web application that downloads YouTube videos, extracts frames, and generates educational comprehension questions for children ages 4-7 using OpenAI's GPT-4 Vision API.
