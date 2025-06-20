from rest_framework import serializers
from mptt.models import TreeForeignKey
from .models import User, UserGroup, UserIdentifierType, UserIdentifier, UserIdentifierNasAuthorization
from django.apps import apps

AuthAttributeGroup = apps.get_model('radius', 'AuthAttributeGroup')
RadiusAttribute = apps.get_model('radius', 'RadiusAttribute')

class UserGroupSerializer(serializers.ModelSerializer):
    """
    Serializer for the UserGroup model.
    """
    allow_any_nas = serializers.BooleanField()


    class Meta:
        model = UserGroup
        fields = ['id', 'name', 'description',
                  'allow_any_nas',
                  'parent', 'created_at', 'updated_at']
        read_only_fields = ['created_at', 'updated_at']


class UserGroupTreeSerializer(serializers.ModelSerializer):
    """
    Serializer for the UserGroup model with nested children.
    """
    children = serializers.SerializerMethodField()
    allow_any_nas = serializers.BooleanField()


    class Meta:
        model = UserGroup
        fields = ['id', 'name', 'description', 'parent', 'children', 'allow_any_nas']

    def get_children(self, obj):
        return UserGroupTreeSerializer(obj.get_children(), many=True).data


class UserIdentifierTypeSerializer(serializers.ModelSerializer):
    class Meta:
        model = UserIdentifierType
        fields = ['id', 'name', 'code', 'description', 'created_at', 'updated_at']
        read_only_fields = ['created_at', 'updated_at']



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
        fields = ['id', 'name', 'description', 'created_at', 'updated_at', 'attributes']
        read_only_fields = ['created_at', 'updated_at']



class UserIdentifierSerializer(serializers.ModelSerializer):
    identifier_type = UserIdentifierTypeSerializer(read_only=True)
    identifier_type_id = serializers.PrimaryKeyRelatedField(
        queryset=UserIdentifierType.objects.all(),
        source='identifier_type',
    )
    auth_attribute_group = serializers.SerializerMethodField()
    auth_attribute_group_id = serializers.PrimaryKeyRelatedField(
        queryset=apps.get_model('radius', 'AuthAttributeGroup').objects.all(),
        source='auth_attribute_group',
        required=False,
        allow_null=True
    )
    plain_password = serializers.CharField(
        required=False,
        allow_blank=True,
        allow_null=True,
        help_text="Plain text password for the identifier, will be hashed before saving"
    )
    expired_auth_attribute_group = serializers.SerializerMethodField()
    expired_auth_attribute_group_id = serializers.PrimaryKeyRelatedField(
        queryset=apps.get_model('radius', 'AuthAttributeGroup').objects.all(),
        source='expired_auth_attribute_group',
        required=False,
        allow_null=True
    )
    is_expired = serializers.SerializerMethodField()

    class Meta:
        model = UserIdentifier
        fields = [
            'id', 'identifier_type', 'identifier_type_id', 'value',
            'is_enabled', 'comment', 'auth_attribute_group', 'auth_attribute_group_id',
            'expiration_date', 'reject_expired', 'expired_auth_attribute_group',
            'expired_auth_attribute_group_id', 'created_at', 'updated_at',
            'is_expired', 'plain_password'
        ]
        read_only_fields = ['created_at', 'updated_at']

    def get_is_expired(self, obj):
        return obj.is_expired()

    def get_auth_attribute_group(self, obj):
        if obj.auth_attribute_group:
            return AuthAttributeGroupSerializer(obj.auth_attribute_group).data
        return None

    def get_expired_auth_attribute_group(self, obj):
        if obj.expired_auth_attribute_group:
            return AuthAttributeGroupSerializer(obj.expired_auth_attribute_group).data
        return None

    def validate(self, data):
        if data.get('expiration_date') and not data.get('reject_expired'):
            if not data.get('expired_auth_attribute_group'):
                raise serializers.ValidationError(
                    "Expired attribute group is required when not rejecting expired identifiers"
                )
        return data


class UserSerializer(serializers.ModelSerializer):
    """
    Serializer for the User model.
    """
    full_name = serializers.ReadOnlyField()
    groups = UserGroupSerializer(many=True, read_only=True)
    group_ids = serializers.PrimaryKeyRelatedField(
        queryset=UserGroup.objects.all(),
        write_only=True,
        source='groups',
        many=True,
        required=False
    )
    identifiers = UserIdentifierSerializer(many=True, read_only=True)
    allow_any_nas = serializers.BooleanField(allow_null=True)

    allowed_by_any_nas = serializers.SerializerMethodField(read_only=True)

    def get_allowed_by_any_nas(self, obj):
        if obj.allow_any_nas is not None:
            return obj.allowed_by_any_nas
        for each_group in obj.groups.all():
            if each_group.allow_any_nas:
                return True
        return False

    class Meta:
        model = User
        fields = ['id', 'email', 'first_name', 'last_name', 'full_name', 'phone_number', 'external_id',
                  'is_active', 'groups', 'group_ids', 'created_at', 'updated_at', 'last_login',
                  'identifiers', 'allow_any_nas', 'allowed_by_any_nas']
        read_only_fields = ['created_at', 'updated_at', 'last_login']


class UserCreateSerializer(serializers.ModelSerializer):
    """
    Serializer for creating a new user.
    """
    group_ids = serializers.PrimaryKeyRelatedField(
        queryset=UserGroup.objects.all(),
        write_only=True,
        source='groups',
        many=True,
        required=False
    )
    allow_any_nas = serializers.BooleanField(allow_null=True)


    class Meta:
        model = User
        fields = ['email', 'first_name', 'last_name', 'phone_number',
                  'external_id', 'email',
                  'is_active', 'group_ids', 'allow_any_nas']


class UserUpdateSerializer(serializers.ModelSerializer):
    """
    Serializer for updating an existing user.
    """
    group_ids = serializers.PrimaryKeyRelatedField(
        queryset=UserGroup.objects.all(),
        write_only=True,
        source='groups',
        many=True,
        required=False
    )
    allow_any_nas = serializers.BooleanField(allow_null=True)


    class Meta:
        model = User
        fields = ['first_name',
                  'external_id',
                  'allow_any_nas',
                  'email',
                  'last_name', 'phone_number', 'is_active', 'group_ids']


class UserIdentifierNasAuthorizationSerializer(serializers.ModelSerializer):
    nas_name = serializers.CharField(source='nas.name', read_only=True)
    attribute_group_name = serializers.CharField(source='attribute_group.name', read_only=True)

    class Meta:
        model = UserIdentifierNasAuthorization
        fields = ['id', 'nas', 'nas_name', 'attribute_group',
                  'nas_id', 'attribute_group_id',
                  'attribute_group_name', 'created_at', 'updated_at']
        read_only_fields = ['created_at', 'updated_at']


class UserIdentifierNasAuthorizationCreateSerializer(serializers.ModelSerializer):
    class Meta:
        model = UserIdentifierNasAuthorization
        fields = ['nas', 'attribute_group', 'nas_id', 'attribute_group_id']


class UserIdentifierNasAuthorizationUpdateSerializer(serializers.ModelSerializer):
    class Meta:
        model = UserIdentifierNasAuthorization
        fields = ['attribute_group', 'attribute_group_id']
