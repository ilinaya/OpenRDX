from django.db import models
from django.utils.translation import gettext_lazy as _


class Setting(models.Model):
    """
    Model representing a system setting.
    """
    KEY_TYPES = (
        ('str', _('String')),
        ('int', _('Integer')),
        ('float', _('Float')),
        ('bool', _('Boolean')),
        ('json', _('JSON')),
    )

    key = models.CharField(_("Key"), max_length=255, unique=True)
    value = models.TextField(_("Value"))
    value_type = models.CharField(_("Value Type"), max_length=10, choices=KEY_TYPES, default='str')
    description = models.TextField(_("Description"), blank=True)
    is_public = models.BooleanField(_("Is Public"), default=False, 
                                   help_text=_("If enabled, this setting will be available to unauthenticated users"))
    created_at = models.DateTimeField(_("Created At"), auto_now_add=True)
    updated_at = models.DateTimeField(_("Updated At"), auto_now=True)

    class Meta:
        verbose_name = _("Setting")
        verbose_name_plural = _("Settings")
        ordering = ['key']
        db_table = 'settings_app_setting'

    def __str__(self):
        return f"{self.key}: {self.value}"

    def get_typed_value(self):
        """
        Returns the value converted to its specified type.
        """
        if self.value_type == 'str':
            return self.value
        elif self.value_type == 'int':
            return int(self.value)
        elif self.value_type == 'float':
            return float(self.value)
        elif self.value_type == 'bool':
            return self.value.lower() in ('true', 'yes', '1', 't', 'y')
        elif self.value_type == 'json':
            import json
            return json.loads(self.value)
        return self.value
