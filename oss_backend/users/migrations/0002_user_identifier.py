from django.db import migrations, models
import django.db.models.deletion


class Migration(migrations.Migration):

    dependencies = [
        ('users', '0001_initial'),
        ('radius', '0001_initial'),
    ]

    operations = [
        migrations.CreateModel(
            name='UserIdentifierType',
            fields=[
                ('id', models.BigAutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('name', models.CharField(max_length=100, unique=True)),
                ('description', models.TextField(blank=True)),
                ('created_at', models.DateTimeField(auto_now_add=True)),
                ('updated_at', models.DateTimeField(auto_now=True)),
            ],
            options={
                'verbose_name': 'User Identifier Type',
                'verbose_name_plural': 'User Identifier Types',
                'ordering': ['name'],
            },
        ),
        migrations.CreateModel(
            name='UserIdentifier',
            fields=[
                ('id', models.BigAutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('value', models.CharField(max_length=255)),
                ('is_enabled', models.BooleanField(default=True)),
                ('comment', models.TextField(blank=True)),
                ('expiration_date', models.DateTimeField(blank=True, null=True)),
                ('reject_expired', models.BooleanField(default=False)),
                ('created_at', models.DateTimeField(auto_now_add=True)),
                ('updated_at', models.DateTimeField(auto_now=True)),
                ('auth_attribute_group', models.ForeignKey(blank=True, null=True, on_delete=django.db.models.deletion.SET_NULL, to='radius.authattributegroup')),
                ('expired_auth_attribute_group', models.ForeignKey(blank=True, null=True, on_delete=django.db.models.deletion.SET_NULL, related_name='expired_identifiers', to='radius.authattributegroup')),
                ('identifier_type', models.ForeignKey(on_delete=django.db.models.deletion.PROTECT, to='users.useridentifiertype')),
                ('user', models.ForeignKey(on_delete=django.db.models.deletion.CASCADE, related_name='identifiers', to='users.user')),
            ],
            options={
                'verbose_name': 'User Identifier',
                'verbose_name_plural': 'User Identifiers',
                'ordering': ['-created_at'],
                'unique_together': {('user', 'identifier_type', 'value')},
            },
        ),
    ]