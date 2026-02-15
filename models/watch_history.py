from sqlalchemy import Column, Integer, String, ForeignKey, DateTime
from sqlalchemy.orm import relationship
from datetime import datetime
from db import Base

class WatchHistory(Base):
    __tablename__ = "watch_histories"

    id = Column(Integer, primary_key=True)
    user_id = Column(Integer, ForeignKey("users.id", ondelete="CASCADE"))
    video_id = Column(Integer, ForeignKey("videos.id", ondelete="CASCADE"))
    watched_on = Column(DateTime, default=datetime.utcnow)

    user = relationship("User", back_populates="watch_history")
    video = relationship("Video")
