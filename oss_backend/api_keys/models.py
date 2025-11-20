from django.db import models
from django.utils.translation import gettext_lazy as _
from django.utils import timezone
from django.conf import settings


class ApiKey(models.Model):
    """
    Model representing an API key for programmatic access to the API.
    """
    name = models.CharField(_("Name"), max_length=255, help_text=_("A descriptive name for this API key"))
    key = models.TextField(_("API Key"), help_text=_("The generated JWT API key"))
    expires_at = models.DateTimeField(_("Expires At"), help_text=_("When this API key expires"))
    created_by = models.ForeignKey(
        settings.AUTH_USER_MODEL,
        on_delete=models.SET_NULL,
        null=True,
        blank=True,
        related_name="created_api_keys",
        verbose_name=_("Created By")
    )
    is_active = models.BooleanField(_("Is Active"), default=True, help_text=_("Whether this API key is active"))
    last_used_at = models.DateTimeField(_("Last Used At"), null=True, blank=True, help_text=_("When this API key was last used"))
    created_at = models.DateTimeField(_("Created At"), auto_now_add=True)
    updated_at = models.DateTimeField(_("Updated At"), auto_now=True)

    class Meta:
        verbose_name = _("API Key")
        verbose_name_plural = _("API Keys")
        ordering = ['-created_at']
        db_table = 'api_keys_apikey'
        indexes = [
            models.Index(fields=['created_by'], name='api_key_created_by_idx'),
            models.Index(fields=['is_active'], name='api_key_is_active_idx'),
            models.Index(fields=['expires_at'], name='api_key_expires_at_idx'),
        ]

    def __str__(self):
        return f"{self.name} ({self.created_by.email if self.created_by else 'Unknown'})"

    def is_expired(self):
        """Check if the API key has expired."""
        return timezone.now() > self.expires_at

    def is_valid(self):
        """Check if the API key is valid (active and not expired)."""
        return self.is_active and not self.is_expired()

