from rest_framework import serializers
from django.apps import apps

from shared.serializers import TimezoneSerializer
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
    vendor_id = serializers.IntegerField(
        required=False
    )
    timezone_id = serializers.IntegerField()

    class Meta:
        model = Nas
        fields = ['id', 'name', 'description', 'ip_address', 'coa_enabled', 'coa_port', 
                 'groups', 'group_ids', 'created_at', 'updated_at', 'is_active',
                  'vendor', 'vendor_id', 'timezone_id', 'timezone']

        read_only_fields = ['created_at', 'updated_at', 'timezone']


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

    class Meta:
        model = Nas
        fields = ['name', 'description', 'ip_address', 'coa_enabled', 'coa_port', 
                 'group_ids', 'is_active', 'timezone_id', 'vendor_id']


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


    class Meta:
        model = Nas
        fields = ['name', 'description', 'ip_address', 'coa_enabled', 'coa_port', 
                 'group_ids', 'is_active', 'vendor_id', 'timezone_id']
