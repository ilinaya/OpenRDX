from django.db import models
from django.utils.translation import gettext_lazy as _
from users.models import User


class AuthAttributeGroup(models.Model):
    """
    Model representing a group of RADIUS authentication attributes.
    These groups can be assigned to user-NAS relationships.
    """
    name = models.CharField(_("Group Name"), max_length=255, unique=True)
    description = models.TextField(_("Description"), blank=True)
    is_system = models.BooleanField(_("System Group"), default=False, 
                                   help_text=_("System groups cannot be deleted"))
    created_at = models.DateTimeField(_("Created At"), auto_now_add=True)
    updated_at = models.DateTimeField(_("Updated At"), auto_now=True)

    class Meta:
        verbose_name = _("Auth Attribute Group")
        verbose_name_plural = _("Auth Attribute Groups")
        ordering = ['name']
        db_table = 'radius_auth_attribute_group'

    def __str__(self):
        return self.name


class RadiusAttribute(models.Model):
    """
    Model representing a RADIUS attribute (AVP) that can be included in an authentication response.
    """
    ATTRIBUTE_TYPES = (
        ('string', _('String')),
        ('integer', _('Integer')),
        ('ipaddr', _('IP Address')),
        ('date', _('Date')),
        ('octets', _('Octets')),
    )

    group = models.ForeignKey(AuthAttributeGroup, on_delete=models.CASCADE, 
                             related_name='attributes', verbose_name=_("Attribute Group"))
    vendor_id = models.PositiveIntegerField(_("Vendor ID"), default=0, 
                                           help_text=_("0 for standard attributes"))
    attribute_id = models.PositiveIntegerField(_("Attribute ID"))
    attribute_name = models.CharField(_("Attribute Name"), max_length=255)
    attribute_type = models.CharField(_("Attribute Type"), max_length=20, choices=ATTRIBUTE_TYPES)
    attribute_value = models.CharField(_("Attribute Value"), max_length=255)
    created_at = models.DateTimeField(_("Created At"), auto_now_add=True)
    updated_at = models.DateTimeField(_("Updated At"), auto_now=True)

    class Meta:
        verbose_name = _("RADIUS Attribute")
        verbose_name_plural = _("RADIUS Attributes")
        ordering = ['vendor_id', 'attribute_id']
        unique_together = [['group', 'vendor_id', 'attribute_id']]
        db_table = 'radius_radius_attribute'

    def __str__(self):
        return f"{self.attribute_name} ({self.vendor_id}:{self.attribute_id})"


class UserNasRelationship(models.Model):
    """
    Model representing the relationship between a user and a NAS device,
    including the authentication attribute group assigned to this relationship.
    """
    user = models.ForeignKey(User, on_delete=models.CASCADE, 
                            related_name='nas_relationships', verbose_name=_("User"))
    nas = models.ForeignKey('nas.Nas', on_delete=models.CASCADE, 
                           related_name='user_relationships', verbose_name=_("NAS Device"))
    attribute_group = models.ForeignKey(AuthAttributeGroup, on_delete=models.PROTECT, 
                                       related_name='user_nas_relationships', 
                                       verbose_name=_("Auth Attribute Group"))
    created_at = models.DateTimeField(_("Created At"), auto_now_add=True)
    updated_at = models.DateTimeField(_("Updated At"), auto_now=True)

    # User-specific attribute overrides stored as JSON
    attribute_overrides = models.JSONField(_("Attribute Overrides"), default=dict, blank=True,
                                         help_text=_("User-specific attribute values that override group values"))

    class Meta:
        verbose_name = _("User-NAS Relationship")
        verbose_name_plural = _("User-NAS Relationships")
        unique_together = [['user', 'nas']]
        db_table = 'radius_user_nas_relationship'

    def __str__(self):
        return f"{self.user} -> {self.nas} ({self.attribute_group})"


class Secret(models.Model):
    """
    Model representing a RADIUS secret used for authentication.
    """
    name = models.CharField(_("Name"), max_length=255, unique=True)
    secret = models.CharField(_("Secret"), max_length=255)
    rad_sec = models.BooleanField(_("RADSEC"), default=False, 
                                help_text=_("Whether this secret is used for RADSEC"))
    description = models.TextField(_("Description"), blank=True)
    source_subnets = models.JSONField(_("Source Subnets"), default=list, blank=True,
                                    help_text=_("List of source subnets allowed to use this secret"))
    created_at = models.DateTimeField(_("Created At"), auto_now_add=True)
    updated_at = models.DateTimeField(_("Updated At"), auto_now=True)

    class Meta:
        verbose_name = _("Secret")
        verbose_name_plural = _("Secrets")
        ordering = ['name']
        db_table = 'radius_secret'

    def __str__(self):
        return self.name

