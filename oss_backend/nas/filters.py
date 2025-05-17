import django_filters

from nas.models import Vendor


class VendorFilter(django_filters.FilterSet):
    """
    Filter class for Vendor model.
    """
    name = django_filters.CharFilter(lookup_expr='icontains')
    description = django_filters.CharFilter(lookup_expr='icontains')
    vendor_id = django_filters.NumberFilter(lookup_expr='exact')

    class Meta:
        model = Vendor
        fields = [
            'name',
            'description',
            'vendor_id',
        ]