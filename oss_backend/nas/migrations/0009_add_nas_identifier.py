# Generated migration to add nas_identifier field

from django.db import migrations, models


def set_default_nas_identifier(apps, schema_editor):
    """Set default nas_identifier for existing records using name"""
    Nas = apps.get_model('nas', 'Nas')
    for nas in Nas.objects.filter(nas_identifier__isnull=True):
        nas.nas_identifier = nas.name
        nas.save()


def reverse_set_default_nas_identifier(apps, schema_editor):
    """Reverse migration - no action needed"""
    pass


class Migration(migrations.Migration):

    dependencies = [
        ('nas', '0008_change_ip_address_to_charfield'),
    ]

    operations = [
        migrations.AddField(
            model_name='nas',
            name='nas_identifier',
            field=models.CharField(max_length=255, null=True, blank=True, verbose_name='NAS Identifier'),
        ),
        migrations.RunPython(
            set_default_nas_identifier,
            reverse_set_default_nas_identifier,
        ),
        migrations.AlterField(
            model_name='nas',
            name='nas_identifier',
            field=models.CharField(max_length=255, verbose_name='NAS Identifier'),
        ),
        migrations.AddIndex(
            model_name='nas',
            index=models.Index(fields=['nas_identifier'], name='nas_nas_identifier_idx'),
        ),
    ]

