from django.db import models
from django.utils.translation import gettext_lazy as _
from django.db.models.signals import post_migrate
from django.dispatch import receiver


class RadsecSource(models.Model):
    """
    Model representing a NAS device vendor.
    """
    name = models.CharField(_("Vendor Name"), max_length=255)
    description = models.TextField(_("Description"), blank=True)
    created_at = models.DateTimeField(_("Created At"), auto_now_add=True)
    updated_at = models.DateTimeField(_("Updated At"), auto_now=True)
    source_subnets = models.JSONField(_("Source Subnets"), default=list, blank=True,
                                      help_text=_("List of source subnets allowed to use this secret"))
    tls_key = models.TextField(_("TLS Key"), blank=False, null=False)
    tls_cert = models.TextField(_("TLS Cert"), blank=False, null=False)

    class Meta:
        verbose_name = _("RadSec Source")
        verbose_name_plural = _("RadSec Sources")
        ordering = ['name']
        db_table = 'radec_sources'



class RadsecDestination(models.Model):
    name = models.CharField(_("Vendor Name"), max_length=255)
    description = models.TextField(_("Description"), blank=True)
    created_at = models.DateTimeField(_("Created At"), auto_now_add=True)
    updated_at = models.DateTimeField(_("Updated At"), auto_now=True)
    radsec_source = models.ForeignKey(RadsecSource, on_delete=models.CASCADE)

    class Meta:
        verbose_name = _("RadSec Destination")
        verbose_name_plural = _("RadSec Destinations")
        ordering = ['name']
        indexes = [
            models.Index(fields=['radsec_source'], name='rad_sec_source_idx'),
        ]

        db_table = 'radsec_destinations'