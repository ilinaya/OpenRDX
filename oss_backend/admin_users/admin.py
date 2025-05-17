from django.contrib import admin
from django.contrib.auth.admin import UserAdmin
from .models import AdminUser


@admin.register(AdminUser)
class AdminUserAdmin(UserAdmin):
    """
    Admin configuration for the AdminUser model.
    """
    list_display = ('username', 'email', 'first_name', 'last_name', 'position', 'is_active', 'is_staff', 'is_superuser')
    list_filter = ('is_active', 'is_staff', 'is_superuser')
    search_fields = ('username', 'email', 'first_name', 'last_name', 'position')
    ordering = ('username',)
    
    fieldsets = (
        (None, {'fields': ('username', 'password')}),
        ('Personal info', {'fields': ('first_name', 'last_name', 'email', 'phone_number', 'position')}),
        ('Permissions', {'fields': ('is_active', 'is_staff', 'is_superuser', 'groups', 'user_permissions')}),
        ('Important dates', {'fields': ('last_login', 'date_joined')}),
    )
    
    add_fieldsets = (
        (None, {
            'classes': ('wide',),
            'fields': ('username', 'email', 'password1', 'password2', 'first_name', 'last_name', 'phone_number', 'position'),
        }),
    )