# Generated by Django 5.2.1 on 2025-05-20 22:57

from django.db import migrations


class Migration(migrations.Migration):

    dependencies = [
        ('nas', '0004_nas_timezone'),
    ]

    operations = [
        migrations.RemoveField(
            model_name='nas',
            name='secret',
        ),
    ]
