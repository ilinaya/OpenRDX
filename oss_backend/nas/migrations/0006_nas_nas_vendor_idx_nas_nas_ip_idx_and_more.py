# Generated by Django 5.2.1 on 2025-06-01 18:02

from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ('nas', '0005_remove_nas_secret'),
        ('shared', '0003_alter_timezone_table'),
    ]

    operations = [
        migrations.AddIndex(
            model_name='nas',
            index=models.Index(fields=['vendor'], name='nas_vendor_idx'),
        ),
        migrations.AddIndex(
            model_name='nas',
            index=models.Index(fields=['ip_address'], name='nas_ip_idx'),
        ),
        migrations.AddIndex(
            model_name='nasgroup',
            index=models.Index(fields=['parent'], name='nas_group_parent_idx'),
        ),
        migrations.AddIndex(
            model_name='vendorattribute',
            index=models.Index(fields=['vendor'], name='attrs_vendor_idx'),
        ),
    ]
