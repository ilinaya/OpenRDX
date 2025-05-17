from django.db import models
from django.utils.translation import gettext_lazy as _
from mptt.models import MPTTModel, TreeForeignKey
from django.db.models.signals import post_migrate
from django.dispatch import receiver


class UserGroup(MPTTModel):
    """
    Model representing a group of users with tree structure.
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
        verbose_name = _("User Group")
        verbose_name_plural = _("User Groups")
        db_table = 'users_user_group'

    def __str__(self):
        return self.name


class User(models.Model):
    """
    Model representing a regular user in the system.
    """
    email = models.EmailField(_("Email"), unique=True)
    first_name = models.CharField(_("First Name"), max_length=150, blank=True)
    last_name = models.CharField(_("Last Name"), max_length=150, blank=True)
    phone_number = models.CharField(_("Phone Number"), max_length=20, blank=True)
    is_active = models.BooleanField(_("Is Active"), default=True)
    groups = models.ManyToManyField(UserGroup, related_name="users", blank=True, 
                                   verbose_name=_("User Groups"))
    created_at = models.DateTimeField(_("Created At"), auto_now_add=True)
    updated_at = models.DateTimeField(_("Updated At"), auto_now=True)
    last_login = models.DateTimeField(_("Last Login"), null=True, blank=True)

    class Meta:
        verbose_name = _("User")
        verbose_name_plural = _("Users")
        ordering = ['-created_at']
        db_table = 'users_user'

    def __str__(self):
        return self.email

    @property
    def full_name(self):
        """
        Return the full name of the user.
        """
        if self.first_name and self.last_name:
            return f"{self.first_name} {self.last_name}"
        return self.email

