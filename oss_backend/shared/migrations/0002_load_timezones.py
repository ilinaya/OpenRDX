from django.db import migrations
import json
import os


def load_timezones(apps, schema_editor):
    Timezone = apps.get_model('shared', 'Timezone')
    json_file = os.path.join(os.path.dirname(__file__), '../data/timezones.json')
    
    with open(json_file) as f:
        data = json.load(f)
        for tz in data['timezones']:
            Timezone.objects.create(
                name=tz['name'],
                offset=tz['offset']
            )


def reverse_load_timezones(apps, schema_editor):
    Timezone = apps.get_model('shared', 'Timezone')
    Timezone.objects.all().delete()


class Migration(migrations.Migration):

    dependencies = [
        ('shared', '0001_initial'),
    ]

    operations = [
        migrations.RunPython(load_timezones, reverse_load_timezones),
    ] 