from rest_framework import serializers
from .models import AdminUser, AdminGroup


class AdminUserSerializer(serializers.ModelSerializer):
    """
    Serializer for the AdminUser model.
    """
    password = serializers.CharField(write_only=True, required=False)

    class Meta:
        model = AdminUser
        fields = ('id', 'username', 'email', 'first_name', 'last_name', 
                  'phone_number', 'position', 'is_active', 'is_staff', 
                  'is_superuser', 'password', 'date_joined', 'last_login')
        read_only_fields = ('id', 'date_joined', 'last_login')

    def create(self, validated_data):
        """
        Create and return a new AdminUser instance, given the validated data.
        """
        password = validated_data.pop('password', None)
        user = AdminUser.objects.create(**validated_data)

        if password:
            user.set_password(password)
            user.save()

        return user

    def update(self, instance, validated_data):
        """
        Update and return an existing AdminUser instance, given the validated data.
        """
        password = validated_data.pop('password', None)

        for attr, value in validated_data.items():
            setattr(instance, attr, value)

        if password:
            instance.set_password(password)

        instance.save()
        return instance


class AdminUserCreateSerializer(AdminUserSerializer):
    """
    Serializer for creating AdminUser instances.
    """
    password = serializers.CharField(write_only=True, required=True)

    class Meta(AdminUserSerializer.Meta):
        fields = ('id', 'username', 'email', 'first_name', 'last_name', 
                  'phone_number', 'position', 'is_active', 'is_staff', 
                  'is_superuser', 'password')


class AdminGroupSerializer(serializers.ModelSerializer):
    """
    Serializer for the AdminGroup model.
    """
    class Meta:
        model = AdminGroup
        fields = ('id', 'name', 'description', 'created_at', 'updated_at')
        read_only_fields = ('id', 'created_at', 'updated_at')


class AdminGroupCreateSerializer(AdminGroupSerializer):
    """
    Serializer for creating AdminGroup instances.
    """
    class Meta(AdminGroupSerializer.Meta):
        fields = ('id', 'name', 'description')
