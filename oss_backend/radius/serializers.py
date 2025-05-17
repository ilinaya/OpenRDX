from rest_framework import serializers
from .models import AuthAttributeGroup, RadiusAttribute, UserNasRelationship, Secret
from users.serializers import UserSerializer



class RadiusAttributeSerializer(serializers.ModelSerializer):
    """
    Serializer for the RadiusAttribute model.
    """
    class Meta:
        model = RadiusAttribute
        fields = ['id', 'group', 'vendor_id', 'attribute_id', 'attribute_name', 
                 'attribute_type', 'attribute_value', 'created_at', 'updated_at']
        read_only_fields = ['created_at', 'updated_at']


class AuthAttributeGroupSerializer(serializers.ModelSerializer):
    """
    Serializer for the AuthAttributeGroup model.
    """
    attributes = RadiusAttributeSerializer(many=True, read_only=True)

    class Meta:
        model = AuthAttributeGroup
        fields = ['id', 'name', 'description', 'is_system', 'attributes', 
                 'created_at', 'updated_at']
        read_only_fields = ['is_system', 'created_at', 'updated_at']


class AuthAttributeGroupCreateSerializer(serializers.ModelSerializer):
    """
    Serializer for creating a new AuthAttributeGroup.
    """
    class Meta:
        model = AuthAttributeGroup
        fields = ['id', 'name', 'description']


class RadiusAttributeCreateSerializer(serializers.ModelSerializer):
    """
    Serializer for creating a new RadiusAttribute.
    """
    class Meta:
        model = RadiusAttribute
        fields = ['group', 'vendor_id', 'attribute_id', 'attribute_name', 
                 'attribute_type', 'attribute_value']


class UserNasRelationshipSerializer(serializers.ModelSerializer):
    """
    Serializer for the UserNasRelationship model.
    """
    user_details = UserSerializer(source='user', read_only=True)
    attribute_group_details = AuthAttributeGroupSerializer(source='attribute_group', read_only=True)

    def to_representation(self, instance):
        """
        Override to_representation to add nas_details using lazy import
        """
        from nas.serializers import NasSerializer

        ret = super().to_representation(instance)
        ret['nas_details'] = NasSerializer(instance.nas).data
        return ret

    class Meta:
        model = UserNasRelationship
        fields = ['id', 'user', 'nas', 'attribute_group', 'attribute_overrides', 
                 'user_details', 'attribute_group_details',
                 'created_at', 'updated_at']
        read_only_fields = ['created_at', 'updated_at']


class UserNasRelationshipCreateSerializer(serializers.ModelSerializer):
    """
    Serializer for creating a new UserNasRelationship.
    """
    class Meta:
        model = UserNasRelationship
        fields = ['user', 'nas', 'attribute_group', 'attribute_overrides']


class UserNasRelationshipUpdateSerializer(serializers.ModelSerializer):
    """
    Serializer for updating an existing UserNasRelationship.
    """
    class Meta:
        model = UserNasRelationship
        fields = ['attribute_group', 'attribute_overrides']


class SecretSerializer(serializers.ModelSerializer):
    """
    Serializer for the Secret model.
    """
    class Meta:
        model = Secret
        fields = ['id', 'name', 'secret', 'rad_sec', 'description', 'source_subnets', 
                 'created_at', 'updated_at']
        read_only_fields = ['created_at', 'updated_at']


class SecretCreateSerializer(serializers.ModelSerializer):
    """
    Serializer for creating a new Secret.
    """
    class Meta:
        model = Secret
        fields = ['name', 'secret', 'rad_sec', 'description', 'source_subnets']


class SecretUpdateSerializer(serializers.ModelSerializer):
    """
    Serializer for updating an existing Secret.
    """
    class Meta:
        model = Secret
        fields = ['name', 'secret', 'rad_sec', 'description', 'source_subnets']
