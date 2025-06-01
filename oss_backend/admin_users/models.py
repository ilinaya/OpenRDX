from django.contrib.auth.models import AbstractUser
from django.db import models
from django.utils import timezone
from django.db.models.signals import post_migrate
from django.dispatch import receiver


class AdminGroup(models.Model):
    """
    Model representing a group of admin users.
    """
    name = models.CharField("Group Name", max_length=255)
    description = models.TextField("Description", blank=True)
    created_at = models.DateTimeField("Created At", auto_now_add=True)
    updated_at = models.DateTimeField("Updated At", auto_now=True)

    class Meta:
        verbose_name = 'Admin Group'
        verbose_name_plural = 'Admin Groups'
        db_table = 'admin_users_admin_group'

    def __str__(self):
        return self.name


class AdminUser(AbstractUser):
    """
    Custom user model that extends Django's AbstractUser.

    This model is used for administrative users in the system.
    """
    # Additional fields for admin users
    phone_number = models.CharField(max_length=20, blank=True, null=True)
    position = models.CharField(max_length=100, blank=True, null=True)
    is_active = models.BooleanField(default=True)
    groups = models.ManyToManyField(AdminGroup, related_name="admin_users", blank=True, 
                                   verbose_name="Admin Groups")

    # Override the default is_staff to True for admin users
    is_staff = models.BooleanField(
        default=True,
        help_text='Designates whether the user can log into this admin site.',
    )

    # Fields for invitation and password reset
    invitation_token = models.CharField(max_length=100, blank=True, null=True)
    invitation_expires = models.DateTimeField(blank=True, null=True)
    reset_token = models.CharField(max_length=100, blank=True, null=True)
    reset_expires = models.DateTimeField(blank=True, null=True)
    email = models.EmailField(
        'email address',
        unique=True,
    )

    USERNAME_FIELD = 'email'
    REQUIRED_FIELDS = ['first_name', 'last_name']



    class Meta:
        verbose_name = 'Admin User'
        verbose_name_plural = 'Admin Users'
        indexes = [
            models.Index(fields=['email'], name='admin_email_idx'),
        ]
        db_table = 'admin_users_admin_user'

    def __str__(self):
        return self.username

    def is_invitation_valid(self):
        """Check if the invitation token is still valid."""
        if not self.invitation_token or not self.invitation_expires:
            return False
        return timezone.now() < self.invitation_expires

    def is_reset_token_valid(self):
        """Check if the password reset token is still valid."""
        if not self.reset_token or not self.reset_expires:
            return False
        return timezone.now() < self.reset_expires


