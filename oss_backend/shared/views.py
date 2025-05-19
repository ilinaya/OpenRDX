from rest_framework import viewsets
from rest_framework.pagination import PageNumberPagination
from .models import Timezone
from .serializers import TimezoneSerializer


class NoPagination(PageNumberPagination):
    page_size = None


class TimezoneViewSet(viewsets.ReadOnlyModelViewSet):
    queryset = Timezone.objects.all()
    serializer_class = TimezoneSerializer
    pagination_class = NoPagination 