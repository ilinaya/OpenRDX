from rest_framework import viewsets
from rest_framework.decorators import action
from rest_framework.response import Response
from django_filters.rest_framework import DjangoFilterBackend
from rest_framework.filters import SearchFilter, OrderingFilter
from .models import RadsecSource, RadsecDestination
from .serializers import RadsecSourceSerializer, RadsecSourceCreateSerializer, RadsecSourceUpdateSerializer

class SourcesViewSet(viewsets.ModelViewSet):
    """
    ViewSet for managing RADIUS secrets.
    """
    queryset = RadsecSource.objects.all()
    serializer_class = RadsecSourceSerializer
    filter_backends = [DjangoFilterBackend, SearchFilter, OrderingFilter]
    filterset_fields = ['name']
    search_fields = ['name', 'description']
    ordering_fields = ['name', 'created_at']
    ordering = ['name']

    @action(detail=False, methods=['get'])
    def list_all(self, request):
        """
        Return a list of all secrets without pagination.
        """
        queryset = self.filter_queryset(self.get_queryset())
        serializer = self.get_serializer(queryset, many=True)
        return Response(serializer.data)

    def get_serializer_class(self):
        """
        Return appropriate serializer class based on the action.
        """
        if self.action == 'create':
            return RadsecSourceCreateSerializer
        elif self.action in ['update', 'partial_update']:
            return RadsecSourceUpdateSerializer
        return self.serializer_class