from django.db import models


class Timezone(models.Model):
    name = models.CharField(max_length=100, unique=True)
    offset = models.IntegerField(help_text="Offset in minutes from UTC")
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        ordering = ['offset', 'name']
        verbose_name = 'Timezone'
        verbose_name_plural = 'Timezones'
        db_table = 'timezones'

    def __str__(self):
        return f"{self.name} (UTC{'+' if self.offset >= 0 else ''}{self.offset//60:02d}:{abs(self.offset)%60:02d})" 