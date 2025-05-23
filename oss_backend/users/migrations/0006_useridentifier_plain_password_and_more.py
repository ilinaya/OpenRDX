# Generated by Django 5.2.1 on 2025-05-20 23:35

from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ('radius', '0003_create_secret_model_and_default_secret'),
        ('users', '0005_user_external_id_user_user_email_idx_and_more'),
    ]

    operations = [
        migrations.AddField(
            model_name='useridentifier',
            name='plain_password',
            field=models.CharField(blank=True, max_length=255, null=True),
        ),
        migrations.AddIndex(
            model_name='useridentifier',
            index=models.Index(fields=['value'], name='user_identi_value_e364f2_idx'),
        ),
        migrations.AddIndex(
            model_name='useridentifier',
            index=models.Index(fields=['user'], name='user_identi_user_id_a5310c_idx'),
        ),
    ]
