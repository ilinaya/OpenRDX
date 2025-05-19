from rest_framework import serializers
from .models import Timezone


class TimezoneSerializer(serializers.ModelSerializer):
    offset_formatted = serializers.SerializerMethodField()

    class Meta:
        model = Timezone
        fields = ['id', 'name', 'offset', 'offset_formatted']
        read_only_fields = ['id', 'offset_formatted']

    def get_offset_formatted(self, obj):
        sign = '+' if obj.offset >= 0 else ''
        hours = abs(obj.offset) // 60
        minutes = abs(obj.offset) % 60
        return f"UTC{sign}{hours:02d}:{minutes:02d}" 