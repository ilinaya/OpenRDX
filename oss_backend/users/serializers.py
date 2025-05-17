from rest_framework import serializers
from mptt.models import TreeForeignKey
from .models import User, UserGroup


class UserGroupSerializer(serializers.ModelSerializer):
    """
    Serializer for the UserGroup model.
    """
    class Meta:
        model = UserGroup
        fields = ['id', 'name', 'description', 'parent', 'created_at', 'updated_at']
        read_only_fields = ['created_at', 'updated_at']


class UserGroupTreeSerializer(serializers.ModelSerializer):
    """
    Serializer for the UserGroup model with nested children.
    """
    children = serializers.SerializerMethodField()

    class Meta:
        model = UserGroup
        fields = ['id', 'name', 'description', 'parent', 'children']

    def get_children(self, obj):
        return UserGroupTreeSerializer(obj.get_children(), many=True).data


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

    class Meta:
        model = User
        fields = ['id', 'email', 'first_name', 'last_name', 'full_name', 'phone_number', 
                  'is_active', 'groups', 'group_ids', 'created_at', 'updated_at', 'last_login']
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

    class Meta:
        model = User
        fields = ['email', 'first_name', 'last_name', 'phone_number', 'is_active', 'group_ids']


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

    class Meta:
        model = User
        fields = ['first_name', 'last_name', 'phone_number', 'is_active', 'group_ids']
