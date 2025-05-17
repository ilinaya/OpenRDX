from django.contrib import admin
from .models import AuthAttributeGroup, RadiusAttribute, UserNasRelationship, Secret


class RadiusAttributeInline(admin.TabularInline):
    model = RadiusAttribute
    extra = 1


@admin.register(AuthAttributeGroup)
class AuthAttributeGroupAdmin(admin.ModelAdmin):
    list_display = ('name', 'description', 'is_system', 'created_at')
    list_filter = ('is_system',)
    search_fields = ('name', 'description')
    readonly_fields = ('is_system',)
    inlines = [RadiusAttributeInline]


@admin.register(RadiusAttribute)
class RadiusAttributeAdmin(admin.ModelAdmin):
    list_display = ('attribute_name', 'vendor_id', 'attribute_id', 'attribute_type', 
                   'attribute_value', 'group')
    list_filter = ('group', 'attribute_type', 'vendor_id')
    search_fields = ('attribute_name', 'attribute_value')


@admin.register(UserNasRelationship)
class UserNasRelationshipAdmin(admin.ModelAdmin):
    list_display = ('user', 'nas', 'attribute_group', 'created_at')
    list_filter = ('attribute_group', 'nas')
    search_fields = ('user__email', 'nas__name', 'attribute_group__name')
    raw_id_fields = ('user', 'nas', 'attribute_group')


@admin.register(Secret)
class SecretAdmin(admin.ModelAdmin):
    list_display = ('name', 'secret', 'rad_sec', 'description', 'created_at')
    list_filter = ('rad_sec',)
    search_fields = ('name', 'description')
    readonly_fields = ('created_at', 'updated_at')
