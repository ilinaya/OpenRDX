from django.contrib import admin
from mptt.admin import MPTTModelAdmin
from .models import Nas, NasGroup


@admin.register(NasGroup)
class NasGroupAdmin(MPTTModelAdmin):
    list_display = ('name', 'description', 'parent')
    search_fields = ('name', 'description')
    list_filter = ('parent',)


@admin.register(Nas)
class NasAdmin(admin.ModelAdmin):
    list_display = ('name', 'ip_address', 'coa_enabled', 'coa_port', 'is_active')
    list_filter = ('coa_enabled', 'is_active', 'groups')
    search_fields = ('name', 'description', 'ip_address')
    filter_horizontal = ('groups',)
