# Generated by Django 5.2.1 on 2025-05-21 02:24

import django.db.models.deletion
from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ('nas', '0005_remove_nas_secret'),
        ('radius', '0003_create_secret_model_and_default_secret'),
        ('users', '0011_useridentifiertype_code_and_more'),
    ]

    operations = [
        migrations.CreateModel(
            name='UserIdentifierNasAuthorization',
            fields=[
                ('id', models.BigAutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('created_at', models.DateTimeField(auto_now_add=True)),
                ('updated_at', models.DateTimeField(auto_now=True)),
                ('attribute_group', models.ForeignKey(null=True, on_delete=django.db.models.deletion.SET_NULL, related_name='user_identifier_nas_authorizations', to='radius.authattributegroup')),
                ('nas', models.ForeignKey(on_delete=django.db.models.deletion.CASCADE, related_name='user_identifier_authorizations', to='nas.nas')),
                ('user_identifier', models.ForeignKey(on_delete=django.db.models.deletion.CASCADE, related_name='nas_authorizations', to='users.useridentifier')),
            ],
            options={
                'verbose_name': 'User Identifier NAS Authorization',
                'verbose_name_plural': 'User Identifier NAS Authorizations',
                'db_table': 'user_identifier_authorizations',
                'indexes': [models.Index(fields=['user_identifier'], name='user_identi_user_id_0f64d2_idx'), models.Index(fields=['nas'], name='user_identi_nas_id_3cc2cc_idx')],
                'unique_together': {('user_identifier', 'nas')},
            },
        ),
    ]
