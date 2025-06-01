from rest_framework import serializers
from .models import *


class RadsecSourceSerializer(serializers.ModelSerializer):
    """
    Serializer for the Secret model.
    """
    class Meta:
        model = RadsecSource
        fields = ['id', 'name',  'description', 'source_subnets', 'tls_key', 'tls_cert',
                 'created_at', 'updated_at']
        read_only_fields = ['created_at', 'updated_at']


class RadsecSourceCreateSerializer(serializers.ModelSerializer):
    """
    Serializer for creating a new Secret.
    """
    class Meta:
        model = RadsecSource
        fields = ['name', 'description', 'source_subnets', 'tls_key', 'tls_cert']


class RadsecSourceUpdateSerializer(serializers.ModelSerializer):
    """
    Serializer for updating an existing Secret.
    """
    class Meta:
        model = RadsecSource
        fields = ['name', 'description', 'source_subnets', 'tls_key', 'tls_cert']
