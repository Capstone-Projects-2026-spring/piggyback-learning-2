from sqlalchemy import Column, Integer, String, ForeignKey, DateTime
from sqlalchemy.orm import relationship
from db import Base

class UserProfile(Base):
    __tablename__ = "user_profiles"

    id = Column(Integer, primary_key=True)
    user_id = Column(Integer, ForeignKey("users.id", ondelete="CASCADE"), unique=True)

    first_name = Column(String(60))
    last_name = Column(String(100))
    date_of_birth = Column(DateTime)

    user = relationship("User", back_populates="profile")
