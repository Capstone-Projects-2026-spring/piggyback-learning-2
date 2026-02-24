from rest_framework import serializers


class VerifyPasswordRequestSerializer(serializers.Serializer):
    user_type = serializers.ChoiceField(choices=['admin', 'expert'])
    password = serializers.CharField()


class VerifyPasswordSuccessSerializer(serializers.Serializer):
    success = serializers.BooleanField(default=True)
    redirect = serializers.CharField()


class VerifyPasswordFailureSerializer(serializers.Serializer):
    success = serializers.BooleanField(default=False)
    message = serializers.CharField(default='Invalid password')
