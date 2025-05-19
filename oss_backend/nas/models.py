from django.db import models
from django.utils.translation import gettext_lazy as _
from mptt.models import MPTTModel, TreeForeignKey
from django.db.models.signals import post_migrate
from django.dispatch import receiver


class Vendor(models.Model):
    """
    Model representing a NAS device vendor.
    """
    name = models.CharField(_("Vendor Name"), max_length=255)
    description = models.TextField(_("Description"), blank=True)
    vendor_id = models.PositiveIntegerField(_("Vendor ID"), unique=True)
    created_at = models.DateTimeField(_("Created At"), auto_now_add=True)
    updated_at = models.DateTimeField(_("Updated At"), auto_now=True)

    class Meta:
        verbose_name = _("Vendor")
        verbose_name_plural = _("Vendors")
        ordering = ['name']
        db_table = 'nas_vendor'

    def __str__(self):
        return f"{self.name} ({self.vendor_id})"


class VendorAttribute(models.Model):
    """
    Model representing a vendor-specific RADIUS attribute.
    """
    ATTRIBUTE_TYPES = (
        ('string', _('String')),
        ('integer', _('Integer')),
        ('ipaddr', _('IP Address')),
        ('date', _('Date')),
        ('octets', _('Octets')),
    )

    vendor = models.ForeignKey(Vendor, on_delete=models.CASCADE, 
                              related_name='attributes', verbose_name=_("Vendor"))
    name = models.CharField(_("Attribute Name"), max_length=255)
    description = models.TextField(_("Description"), blank=True)
    attribute_id = models.PositiveIntegerField(_("Attribute ID"))
    attribute_type = models.CharField(_("Attribute Type"), max_length=20, choices=ATTRIBUTE_TYPES)
    created_at = models.DateTimeField(_("Created At"), auto_now_add=True)
    updated_at = models.DateTimeField(_("Updated At"), auto_now=True)

    class Meta:
        verbose_name = _("Vendor Attribute")
        verbose_name_plural = _("Vendor Attributes")
        ordering = ['vendor', 'attribute_id']
        unique_together = [['vendor', 'attribute_id']]
        db_table = 'nas_vendor_attribute'

    def __str__(self):
        return f"{self.name} ({self.vendor.name}:{self.attribute_id})"


class NasGroup(MPTTModel):
    """
    Model representing a group of NAS devices with tree structure.
    """
    name = models.CharField(_("Group Name"), max_length=255)
    description = models.TextField(_("Description"), blank=True)
    parent = TreeForeignKey('self', on_delete=models.CASCADE, null=True, blank=True, 
                           related_name='children', verbose_name=_("Parent Group"))
    created_at = models.DateTimeField(_("Created At"), auto_now_add=True)
    updated_at = models.DateTimeField(_("Updated At"), auto_now=True)

    class MPTTMeta:
        order_insertion_by = ['name']

    class Meta:
        verbose_name = _("NAS Group")
        verbose_name_plural = _("NAS Groups")
        db_table = 'nas_nas_group'

    def __str__(self):
        return self.name


class Nas(models.Model):
    """
    Model representing a Network Access Server (NAS) device.
    """
    name = models.CharField(_("NAS Name"), max_length=255)
    description = models.TextField(_("Description"), blank=True)
    ip_address = models.GenericIPAddressField(_("IP Address"))
    coa_enabled = models.BooleanField(_("CoA Enabled"), default=False)
    coa_port = models.PositiveIntegerField(_("CoA Port"), default=3799)
    groups = models.ManyToManyField(NasGroup, related_name="nas_devices", blank=True, 
                                   verbose_name=_("NAS Groups"))
    vendor = models.ForeignKey(Vendor, on_delete=models.SET_NULL, null=True, blank=True,
                              related_name="nas_devices", verbose_name=_("Vendor"))
    secret = models.ForeignKey('radius.Secret', on_delete=models.PROTECT, null=True, blank=True,
                             related_name="nas_devices", verbose_name=_("Secret"))
    created_at = models.DateTimeField(_("Created At"), auto_now_add=True)
    updated_at = models.DateTimeField(_("Updated At"), auto_now=True)
    is_active = models.BooleanField(_("Is Active"), default=True)
    timezone = models.ForeignKey('shared.Timezone', on_delete=models.SET_NULL, null=True, blank=True)


    class Meta:
        verbose_name = _("NAS")
        verbose_name_plural = _("NAS Devices")
        ordering = ['name']
        db_table = 'nas_nas'

    def __str__(self):
        return f"{self.name} ({self.ip_address})"


# Default NAS groups and vendors are now created in migration 0002_create_default_nas_groups_and_vendors.py
