from sqlalchemy import Column, Integer, String, ForeignKey, DateTime
from sqlalchemy.orm import relationship
from db import Base

class Video(Base):
    __tablename__ = "videos"

    id = Column(Integer, primary_key=True, index=True)
    url = Column(String(2048))
    title = Column(String(300))
    duration_in_seconds = Column(Integer)

    watch_histories = relationship("WatchHistory", back_populates="video")
