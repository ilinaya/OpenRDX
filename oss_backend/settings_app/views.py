from rest_framework import viewsets, permissions
from rest_framework.response import Response
from rest_framework import status
from drf_yasg.utils import swagger_auto_schema
from .models import Setting
from .serializers import SettingSerializer, SettingCreateUpdateSerializer


class IsAuthenticatedOrReadOnlyPublic(permissions.BasePermission):
    """
    Custom permission to allow authenticated users to perform any action,
    but allow unauthenticated users to only read public settings.
    """
    def has_permission(self, request, view):
        # Allow all authenticated users
        if request.user and request.user.is_authenticated:
            return True
        
        # Allow read-only access for unauthenticated users
        if request.method in permissions.SAFE_METHODS:
            return True
            
        return False
    
    def has_object_permission(self, request, view, obj):
        # Allow all authenticated users
        if request.user and request.user.is_authenticated:
            return True
        
        # Allow read-only access for unauthenticated users, but only for public settings
        if request.method in permissions.SAFE_METHODS and obj.is_public:
            return True
            
        return False


class SettingViewSet(viewsets.ModelViewSet):
    """
    ViewSet for viewing and editing Setting instances.
    """
    queryset = Setting.objects.all()
    permission_classes = [IsAuthenticatedOrReadOnlyPublic]
    
    def get_serializer_class(self):
        if self.action in ['create', 'update', 'partial_update']:
            return SettingCreateUpdateSerializer
        return SettingSerializer
    
    def get_queryset(self):
        """
        Filter settings based on authentication status.
        Unauthenticated users can only see public settings.
        """
        if self.request.user and self.request.user.is_authenticated:
            return Setting.objects.all()
        return Setting.objects.filter(is_public=True)
    
    @swagger_auto_schema(
        operation_description="List all settings (or only public settings for unauthenticated users)",
        responses={200: SettingSerializer(many=True)}
    )
    def list(self, request, *args, **kwargs):
        return super().list(request, *args, **kwargs)
    
    @swagger_auto_schema(
        operation_description="Create a new setting",
        request_body=SettingCreateUpdateSerializer,
        responses={201: SettingSerializer()}
    )
    def create(self, request, *args, **kwargs):
        serializer = self.get_serializer(data=request.data)
        serializer.is_valid(raise_exception=True)
        self.perform_create(serializer)
        headers = self.get_success_headers(serializer.data)
        return Response(
            SettingSerializer(instance=serializer.instance).data,
            status=status.HTTP_201_CREATED,
            headers=headers
        )
    
    @swagger_auto_schema(
        operation_description="Retrieve a setting by key",
        responses={200: SettingSerializer()}
    )
    def retrieve(self, request, *args, **kwargs):
        return super().retrieve(request, *args, **kwargs)
    
    @swagger_auto_schema(
        operation_description="Update a setting",
        request_body=SettingCreateUpdateSerializer,
        responses={200: SettingSerializer()}
    )
    def update(self, request, *args, **kwargs):
        partial = kwargs.pop('partial', False)
        instance = self.get_object()
        serializer = self.get_serializer(instance, data=request.data, partial=partial)
        serializer.is_valid(raise_exception=True)
        self.perform_update(serializer)
        return Response(SettingSerializer(instance=instance).data)
    
    @swagger_auto_schema(
        operation_description="Delete a setting",
        responses={204: "No content"}
    )
    def destroy(self, request, *args, **kwargs):
        return super().destroy(request, *args, **kwargs)