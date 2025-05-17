import django_filters
from .models import AdminUser

class AdminUserFilter(django_filters.FilterSet):
    """
    Filter class for AdminUser model.
    """
    username = django_filters.CharFilter(lookup_expr='icontains')
    email = django_filters.CharFilter(lookup_expr='icontains')
    first_name = django_filters.CharFilter(lookup_expr='icontains')
    last_name = django_filters.CharFilter(lookup_expr='icontains')
    phone_number = django_filters.CharFilter(lookup_expr='icontains')
    position = django_filters.CharFilter(lookup_expr='icontains')
    is_active = django_filters.BooleanFilter()
    is_staff = django_filters.BooleanFilter()
    is_superuser = django_filters.BooleanFilter()
    
    class Meta:
        model = AdminUser
        fields = [
            'username', 
            'email', 
            'first_name', 
            'last_name', 
            'phone_number', 
            'position', 
            'is_active', 
            'is_staff', 
            'is_superuser'
        ]