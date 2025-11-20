# Generated manually to add secret field back to NAS model

from django.db import migrations, models
import django.db.models.deletion


class Migration(migrations.Migration):

    dependencies = [
        ('nas', '0006_nas_nas_vendor_idx_nas_nas_ip_idx_and_more'),
        ('radius', '0003_create_secret_model_and_default_secret'),
    ]

    operations = [
        migrations.AddField(
            model_name='nas',
            name='secret',
            field=models.ForeignKey(
                blank=True,
                null=True,
                on_delete=django.db.models.deletion.PROTECT,
                related_name='nas_devices',
                to='radius.secret',
                verbose_name='Secret'
            ),
        ),
    ]

