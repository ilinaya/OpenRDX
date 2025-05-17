from rest_framework import serializers
from mptt.models import TreeForeignKey
from django.apps import apps
from .models import Nas, NasGroup, Vendor

# Get Secret model dynamically to avoid circular imports
Secret = apps.get_model('radius', 'Secret')


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
    secret_id = serializers.PrimaryKeyRelatedField(
        queryset=Secret.objects.all(),
        write_only=True,
        source='secret',
        required=False
    )
    vendor = VendorSerializer(read_only=True)

    class Meta:
        model = Nas
        fields = ['id', 'name', 'description', 'ip_address', 'coa_enabled', 'coa_port', 
                 'groups', 'group_ids', 'secret', 'secret_id', 'created_at', 'updated_at', 'is_active',
                  'vendor', 'vendor_id']

        read_only_fields = ['created_at', 'updated_at']


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
    secret_id = serializers.PrimaryKeyRelatedField(
        queryset=Secret.objects.all(),
        write_only=True,
        source='secret',
        required=False
    )

    class Meta:
        model = Nas
        fields = ['name', 'description', 'ip_address', 'coa_enabled', 'coa_port', 
                 'group_ids', 'secret_id', 'is_active']


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
    secret_id = serializers.PrimaryKeyRelatedField(
        queryset=Secret.objects.all(),
        write_only=True,
        source='secret',
        required=False
    )

    class Meta:
        model = Nas
        fields = ['name', 'description', 'ip_address', 'coa_enabled', 'coa_port', 
                 'group_ids', 'secret_id', 'is_active']
