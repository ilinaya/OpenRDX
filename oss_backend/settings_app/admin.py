from django.contrib import admin
from .models import Setting


@admin.register(Setting)
class SettingAdmin(admin.ModelAdmin):
    list_display = ('key', 'value', 'value_type', 'is_public', 'updated_at')
    list_filter = ('value_type', 'is_public')
    search_fields = ('key', 'value', 'description')
    readonly_fields = ('created_at', 'updated_at')
    fieldsets = (
        (None, {
            'fields': ('key', 'value', 'value_type')
        }),
        ('Options', {
            'fields': ('description', 'is_public')
        }),
        ('Timestamps', {
            'fields': ('created_at', 'updated_at'),
            'classes': ('collapse',)
        }),
    )