from rest_framework import serializers


class TokenObtainSerializer(serializers.Serializer):
    """
    Serializer for obtaining JWT tokens.
    """
    email = serializers.EmailField()
    password = serializers.CharField(write_only=True)