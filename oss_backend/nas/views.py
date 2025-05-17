from rest_framework import viewsets, status
from rest_framework.decorators import action
from rest_framework.response import Response
from django_filters.rest_framework import DjangoFilterBackend
from rest_framework.filters import SearchFilter, OrderingFilter

from .filters import VendorFilter
from .models import Nas, NasGroup, Vendor
from .serializers import (
    NasSerializer, NasCreateSerializer, NasUpdateSerializer,
    NasGroupSerializer, NasGroupTreeSerializer, VendorSerializer
)


class NasGroupViewSet(viewsets.ModelViewSet):
    """
    ViewSet for managing NAS groups.
    """
    queryset = NasGroup.objects.all()
    serializer_class = NasGroupSerializer
    filter_backends = [DjangoFilterBackend, SearchFilter, OrderingFilter]
    filterset_fields = ['name', 'parent']
    search_fields = ['name', 'description']
    ordering_fields = ['name', 'created_at']
    ordering = ['name']

    @action(detail=False, methods=['get'])
    def tree(self, request):
        """
        Return a tree structure of all NAS groups.
        """
        root_nodes = NasGroup.objects.root_nodes()
        serializer = NasGroupTreeSerializer(root_nodes, many=True)
        return Response(serializer.data)


class NasViewSet(viewsets.ModelViewSet):
    """
    ViewSet for managing NAS devices.
    """
    queryset = Nas.objects.all()
    serializer_class = NasSerializer
    filter_backends = [DjangoFilterBackend, SearchFilter, OrderingFilter]
    filterset_fields = ['name', 'ip_address', 'coa_enabled', 'is_active', 'groups']
    search_fields = ['name', 'description', 'ip_address']
    ordering_fields = ['name', 'created_at', 'ip_address']
    ordering = ['name']

    def get_serializer_class(self):
        """
        Return appropriate serializer class based on the request method.
        """
        if self.action == 'create':
            return NasCreateSerializer
        elif self.action in ['update', 'partial_update']:
            return NasUpdateSerializer
        return NasSerializer

    @action(detail=False, methods=['get'])
    def by_group(self, request):
        """
        Filter NAS devices by group ID.
        """
        group_id = request.query_params.get('group_id')
        if not group_id:
            return Response(
                {"error": "group_id parameter is required"},
                status=status.HTTP_400_BAD_REQUEST
            )
        
        try:
            group = NasGroup.objects.get(pk=group_id)
            # Get all descendant groups including the current group
            group_ids = [group.id] + [g.id for g in group.get_descendants()]
            nas_devices = Nas.objects.filter(groups__id__in=group_ids).distinct()
            serializer = self.get_serializer(nas_devices, many=True)
            return Response(serializer.data)
        except NasGroup.DoesNotExist:
            return Response(
                {"error": f"NasGroup with id {group_id} does not exist"},
                status=status.HTTP_404_NOT_FOUND
            )

class VendorsViewSet(viewsets.ModelViewSet):
    """
    ViewSet for viewing and editing AdminUser instances.
    """
    queryset = Vendor.objects.all()
    serializer_class = VendorSerializer
    filterset_class = VendorFilter

    @action(detail=False, methods=['get'])
    def list_all(self, request):
        """
        Return a list of all vendors without pagination.
        """
        queryset = self.filter_queryset(self.get_queryset())
        serializer = self.get_serializer(queryset, many=True)
        return Response(serializer.data)

    def get_serializer_class(self):
        """
        Return the appropriate serializer class based on the action.
        """
        return self.serializer_class

    def create(self, request, *args, **kwargs):
        """
        Create a new vendor.
        """
        serializer = self.get_serializer(data=request.data)
        serializer.is_valid(raise_exception=True)
        self.perform_create(serializer)
        headers = self.get_success_headers(serializer.data)
        return Response(serializer.data, status=status.HTTP_201_CREATED, headers=headers)

    def perform_create(self, serializer):
        """
        Save the user and return the instance.
        """
        return serializer.save()

    def update(self, request, *args, **kwargs):
        """
        Update an existing vendor.
        """
        partial = kwargs.pop('partial', False)
        instance = self.get_object()
        serializer = self.get_serializer(instance, data=request.data, partial=partial)
        serializer.is_valid(raise_exception=True)
        self.perform_update(serializer)

        if getattr(instance, '_prefetched_objects_cache', None):
            # If 'prefetch_related' has been applied to a queryset, we need to
            # forcibly invalidate the prefetch cache on the instance.
            instance._prefetched_objects_cache = {}

        return Response(serializer.data)