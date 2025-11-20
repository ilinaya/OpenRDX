from rest_framework import serializers
from django.apps import apps
import re
import ipaddress

from shared.serializers import TimezoneSerializer
from radius.serializers import SecretSerializer
from .models import Nas, NasGroup, Vendor

class VendorSerializer(serializers.ModelSerializer):
    """
    Serializer for the Vendor model.
    """
    class Meta:
        model = Vendor
        fields = ['id', 'name', 'description', 'created_at', 'updated_at', 'vendor_id']
        read_only_fields = ['id', 'created_at', 'updated_at', 'id']


class NasGroupSerializer(serializers.ModelSerializer):
    """
    Serializer for the NasGroup model.
    """
    class Meta:
        model = NasGroup
        fields = ['id', 'name', 'description', 'parent', 'created_at', 'updated_at']
        read_only_fields = ['id', 'created_at', 'updated_at']


class NasGroupTreeSerializer(serializers.ModelSerializer):
    """
    Serializer for the NasGroup model with nested children.
    """
    children = serializers.SerializerMethodField()

    class Meta:
        model = NasGroup
        fields = ['id', 'name', 'description', 'parent', 'children']

    def get_children(self, obj):
        return NasGroupTreeSerializer(obj.get_children(), many=True).data


class NasSerializer(serializers.ModelSerializer):
    """
    Serializer for the Nas model.
    """
    groups = NasGroupSerializer(many=True, read_only=True)
    group_ids = serializers.PrimaryKeyRelatedField(
        queryset=NasGroup.objects.all(),
        write_only=True,
        source='groups',
        many=True,
        required=False
    )

    vendor = VendorSerializer(read_only=True)
    timezone = TimezoneSerializer(read_only=True)
    secret = SecretSerializer(read_only=True)
    vendor_id = serializers.IntegerField(
        required=False
    )
    timezone_id = serializers.IntegerField()
    secret_id = serializers.SerializerMethodField()

    def get_secret_id(self, obj):
        """Return the secret ID if secret exists, otherwise None"""
        return obj.secret.id if obj.secret else None

    class Meta:
        model = Nas
        fields = ['id', 'name', 'description', 'ip_address', 'coa_enabled', 'coa_port', 
                 'groups', 'group_ids', 'created_at', 'updated_at', 'is_active',
                  'vendor', 'vendor_id', 'timezone_id', 'timezone', 'secret', 'secret_id']

        read_only_fields = ['created_at', 'updated_at', 'timezone', 'secret']


class NasCreateSerializer(serializers.ModelSerializer):
    """
    Serializer for creating a new NAS device.
    """
    group_ids = serializers.PrimaryKeyRelatedField(
        queryset=NasGroup.objects.all(),
        write_only=True,
        source='groups',
        many=True,
        required=False
    )

    timezone_id = serializers.IntegerField(
        required=True
    )
    vendor_id = serializers.IntegerField(
        required=True
    )
    secret_id = serializers.IntegerField(
        required=False,
        allow_null=True
    )

    def validate_ip_address(self, value):
        """Validate that ip_address is either a valid IP address or hostname"""
        if not value:
            return value
        
        # Try to validate as IP address first
        try:
            ipaddress.ip_address(value)
            return value
        except ValueError:
            pass
        
        # If not an IP, validate as hostname
        # Hostname regex: allows letters, numbers, dots, hyphens, underscores
        # Must start and end with alphanumeric character
        hostname_pattern = re.compile(
            r'^([a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?\.)*[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?$'
        )
        
        # Also allow simple hostnames without dots (for localhost, etc.)
        simple_hostname_pattern = re.compile(r'^[a-zA-Z0-9]([a-zA-Z0-9\-_]{0,61}[a-zA-Z0-9])?$')
        
        if hostname_pattern.match(value) or simple_hostname_pattern.match(value):
            return value
        
        raise serializers.ValidationError(
            "IP address must be a valid IPv4 or IPv6 address, or a valid hostname."
        )

    def validate_secret_id(self, value):
        """Validate that the secret exists if provided"""
        if value is not None:
            Secret = apps.get_model('radius', 'Secret')
            try:
                Secret.objects.get(pk=value)
            except Secret.DoesNotExist:
                raise serializers.ValidationError(f"Secret with id {value} does not exist.")
        return value

    def create(self, validated_data):
        """Override create to handle secret_id"""
        secret_id = validated_data.pop('secret_id', None)
        instance = super().create(validated_data)
        if secret_id is not None:
            Secret = apps.get_model('radius', 'Secret')
            instance.secret = Secret.objects.get(pk=secret_id)
            instance.save()
        return instance

    class Meta:
        model = Nas
        fields = ['name', 'description', 'ip_address', 'coa_enabled', 'coa_port', 
                 'group_ids', 'is_active', 'timezone_id', 'vendor_id', 'secret_id']


class NasUpdateSerializer(serializers.ModelSerializer):
    """
    Serializer for updating an existing NAS device.
    """
    group_ids = serializers.PrimaryKeyRelatedField(
        queryset=NasGroup.objects.all(),
        write_only=True,
        source='groups',
        many=True,
        required=False
    )

    vendor_id = serializers.IntegerField(
        required=True
    )
    timezone_id = serializers.IntegerField(
        required=True
    )
    secret_id = serializers.IntegerField(
        required=False,
        allow_null=True
    )

    def validate_ip_address(self, value):
        """Validate that ip_address is either a valid IP address or hostname"""
        if not value:
            return value
        
        # Try to validate as IP address first
        try:
            ipaddress.ip_address(value)
            return value
        except ValueError:
            pass
        
        # If not an IP, validate as hostname
        # Hostname regex: allows letters, numbers, dots, hyphens, underscores
        # Must start and end with alphanumeric character
        hostname_pattern = re.compile(
            r'^([a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?\.)*[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?$'
        )
        
        # Also allow simple hostnames without dots (for localhost, etc.)
        simple_hostname_pattern = re.compile(r'^[a-zA-Z0-9]([a-zA-Z0-9\-_]{0,61}[a-zA-Z0-9])?$')
        
        if hostname_pattern.match(value) or simple_hostname_pattern.match(value):
            return value
        
        raise serializers.ValidationError(
            "IP address must be a valid IPv4 or IPv6 address, or a valid hostname."
        )

    def validate_secret_id(self, value):
        """Validate that the secret exists if provided"""
        if value is not None:
            Secret = apps.get_model('radius', 'Secret')
            try:
                Secret.objects.get(pk=value)
            except Secret.DoesNotExist:
                raise serializers.ValidationError(f"Secret with id {value} does not exist.")
        return value

    def update(self, instance, validated_data):
        """Override update to handle secret_id"""
        secret_id = validated_data.pop('secret_id', None)
        instance = super().update(instance, validated_data)
        if secret_id is not None:
            Secret = apps.get_model('radius', 'Secret')
            instance.secret = Secret.objects.get(pk=secret_id)
            instance.save()
        elif secret_id is None and 'secret_id' in self.initial_data:
            # Handle explicit None/null value
            instance.secret = None
            instance.save()
        return instance

    class Meta:
        model = Nas
        fields = ['name', 'description', 'ip_address', 'coa_enabled', 'coa_port', 
                 'group_ids', 'is_active', 'vendor_id', 'timezone_id', 'secret_id']
