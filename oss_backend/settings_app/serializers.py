from rest_framework import serializers
from .models import Setting


class SettingSerializer(serializers.ModelSerializer):
    """
    Serializer for the Setting model.
    """
    value = serializers.SerializerMethodField()
    
    class Meta:
        model = Setting
        fields = ['id', 'key', 'value', 'value_type', 'description', 'is_public', 'created_at', 'updated_at']
        read_only_fields = ['created_at', 'updated_at']
    
    def get_value(self, obj):
        """
        Return the typed value instead of the raw string.
        """
        return obj.get_typed_value()


class SettingCreateUpdateSerializer(serializers.ModelSerializer):
    """
    Serializer for creating and updating settings.
    """
    class Meta:
        model = Setting
        fields = ['key', 'value', 'value_type', 'description', 'is_public']
        
    def validate(self, data):
        """
        Validate that the value can be converted to the specified type.
        """
        value = data.get('value')
        value_type = data.get('value_type')
        
        if not value_type:
            return data
            
        try:
            if value_type == 'int':
                int(value)
            elif value_type == 'float':
                float(value)
            elif value_type == 'bool':
                # No validation needed, any string can be converted to bool
                pass
            elif value_type == 'json':
                import json
                json.loads(value)
        except (ValueError, TypeError, json.JSONDecodeError):
            raise serializers.ValidationError(f"Value cannot be converted to {value_type}")
            
        return data