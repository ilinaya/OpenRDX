# Generated manually for API Key model

from django.conf import settings
import django.db.models.deletion
from django.db import migrations, models


class Migration(migrations.Migration):

    initial = True

    dependencies = [
        migrations.swappable_dependency(settings.AUTH_USER_MODEL),
    ]

    operations = [
        migrations.CreateModel(
            name='ApiKey',
            fields=[
                ('id', models.BigAutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('name', models.CharField(help_text='A descriptive name for this API key', max_length=255, verbose_name='Name')),
                ('key', models.TextField(help_text='The generated JWT API key', verbose_name='API Key')),
                ('expires_at', models.DateTimeField(help_text='When this API key expires', verbose_name='Expires At')),
                ('is_active', models.BooleanField(default=True, help_text='Whether this API key is active', verbose_name='Is Active')),
                ('last_used_at', models.DateTimeField(blank=True, help_text='When this API key was last used', null=True, verbose_name='Last Used At')),
                ('created_at', models.DateTimeField(auto_now_add=True, verbose_name='Created At')),
                ('updated_at', models.DateTimeField(auto_now=True, verbose_name='Updated At')),
                ('created_by', models.ForeignKey(blank=True, null=True, on_delete=django.db.models.deletion.SET_NULL, related_name='created_api_keys', to=settings.AUTH_USER_MODEL, verbose_name='Created By')),
            ],
            options={
                'verbose_name': 'API Key',
                'verbose_name_plural': 'API Keys',
                'ordering': ['-created_at'],
                'db_table': 'api_keys_apikey',
            },
        ),
        migrations.AddIndex(
            model_name='apikey',
            index=models.Index(fields=['created_by'], name='api_key_created_by_idx'),
        ),
        migrations.AddIndex(
            model_name='apikey',
            index=models.Index(fields=['is_active'], name='api_key_is_active_idx'),
        ),
        migrations.AddIndex(
            model_name='apikey',
            index=models.Index(fields=['expires_at'], name='api_key_expires_at_idx'),
        ),
    ]

