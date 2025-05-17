from django.contrib import admin
from .models import User


@admin.register(User)
class UserAdmin(admin.ModelAdmin):
    list_display = ('email', 'first_name', 'last_name', 'is_active', 'created_at')
    list_filter = ('is_active',)
    search_fields = ('email', 'first_name', 'last_name', 'phone_number')
    readonly_fields = ('created_at', 'updated_at', 'last_login')
    fieldsets = (
        (None, {
            'fields': ('email', 'is_active')
        }),
        ('Personal Information', {
            'fields': ('first_name', 'last_name', 'phone_number')
        }),
        ('Timestamps', {
            'fields': ('created_at', 'updated_at', 'last_login'),
            'classes': ('collapse',)
        }),
    )