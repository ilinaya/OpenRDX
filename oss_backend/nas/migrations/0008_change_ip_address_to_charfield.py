# Generated manually to change ip_address field to accept hostnames

from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ('nas', '0007_add_secret_to_nas'),
    ]

    operations = [
        migrations.AlterField(
            model_name='nas',
            name='ip_address',
            field=models.CharField(max_length=255, verbose_name='IP Address or Hostname'),
        ),
    ]

