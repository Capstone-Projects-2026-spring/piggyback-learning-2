import uuid

from django.contrib.auth.hashers import make_password
from django.db import models


ALLOWED_CHILD_ICON_KEYS = {
    'cat', 'dog', 'rabbit', 'bear', 'fox', 'owl', 'penguin', 'lion',
    'elephant', 'giraffe', 'monkey', 'panda',
}


class Expert(models.Model):
    expert_id = models.CharField(primary_key=True, max_length=64)
    display_name = models.CharField(max_length=255, blank=True, default='')
    password_hash = models.CharField(max_length=255)
    is_active = models.BooleanField(default=True)
    created_at = models.DateTimeField(auto_now_add=True)

    def __str__(self):
        return self.display_name or self.expert_id

    def set_password(self, raw_password: str):
        self.password_hash = make_password(raw_password)

    def to_dict(self):
        return {
            'expert_id': self.expert_id,
            'display_name': self.display_name or self.expert_id,
            'is_active': self.is_active,
        }


class Child(models.Model):
    child_id = models.CharField(
        primary_key=True, max_length=64, default=lambda: str(uuid.uuid4())
    )
    expert = models.ForeignKey(
        Expert,
        on_delete=models.SET_NULL,
        null=True,
        blank=True,
        related_name='children',
    )
    first_name = models.CharField(max_length=128)
    last_name = models.CharField(max_length=128, blank=True, default='')
    icon_key = models.CharField(max_length=64, blank=True, default='')
    is_active = models.BooleanField(default=True)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        unique_together = [('expert', 'first_name', 'last_name')]

    def __str__(self):
        return f'{self.first_name} {self.last_name}'.strip()

    def to_dict(self):
        return {
            'child_id': self.child_id,
            'expert_id': self.expert_id or '',
            'first_name': self.first_name,
            'last_name': self.last_name,
            'icon_key': self.icon_key,
            'is_active': self.is_active,
        }
