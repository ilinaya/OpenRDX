from django.apps import AppConfig


class NasConfig(AppConfig):
    default_auto_field = 'django.db.models.BigAutoField'
    name = 'nas'
    verbose_name = 'Network Access Server'