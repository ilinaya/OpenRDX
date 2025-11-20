from rest_framework import viewsets, permissions, status
from rest_framework.response import Response
from rest_framework.decorators import action
from django.utils import timezone
from datetime import timedelta
from django.conf import settings
import jwt
from .models import ApiKey
from .serializers import ApiKeySerializer, ApiKeyCreateSerializer


class ApiKeyViewSet(viewsets.ModelViewSet):
    """
    ViewSet for managing API keys.
    API keys cannot be updated, only created and deleted.
    """
    queryset = ApiKey.objects.all()
    serializer_class = ApiKeySerializer
    permission_classes = [permissions.IsAuthenticated]
    http_method_names = ['get', 'post', 'delete', 'head', 'options']  # No PUT or PATCH

    def get_queryset(self):
        """Filter API keys by the current user if not superuser."""
        queryset = super().get_queryset()
        if not self.request.user.is_superuser:
            queryset = queryset.filter(created_by=self.request.user)
        return queryset.order_by('-created_at')

    def get_serializer_class(self):
        """Return the appropriate serializer class based on the action."""
        if self.action == 'create':
            return ApiKeyCreateSerializer
        return ApiKeySerializer

    def create(self, request, *args, **kwargs):
        """
        Create a new API key and generate a JWT token.
        """
        serializer = self.get_serializer(data=request.data)
        serializer.is_valid(raise_exception=True)

        # Get API_KEY_JWT_SECRET from settings
        api_key_secret = settings.API_KEY_JWT_SECRET
        if not api_key_secret:
            return Response(
                {'error': 'API_KEY_JWT_SECRET is not configured. Please run: python manage.py generate_api_key_secret'},
                status=status.HTTP_500_INTERNAL_SERVER_ERROR
            )

        # Calculate expiration date
        validity_days = serializer.validated_data['validity_days']
        expires_at = timezone.now() + timedelta(days=validity_days)

        # Create the API key instance first (with placeholder key)
        api_key = ApiKey(
            name=serializer.validated_data['name'],
            expires_at=expires_at,
            created_by=request.user,
            is_active=True,
            key='',  # Temporary placeholder
        )
        api_key.save()

        # Generate JWT token with the actual API key ID
        payload = {
            'api_key_id': api_key.id,
            'type': 'api_key',
            'created_by': request.user.id,
            'name': serializer.validated_data['name'],
            'exp': int(expires_at.timestamp()),
            'iat': int(timezone.now().timestamp()),
        }

        # Generate JWT token (PyJWT returns string in newer versions)
        token = jwt.encode(payload, api_key_secret, algorithm='HS256')
        
        # Ensure token is a string
        if isinstance(token, bytes):
            token = token.decode('utf-8')
        
        # Update the API key with the generated token
        api_key.key = token
        api_key.save()

        # Return the created API key with the token (only shown once)
        response_serializer = ApiKeySerializer(api_key)
        return Response(response_serializer.data, status=status.HTTP_201_CREATED)

    def update(self, request, *args, **kwargs):
        """
        API keys cannot be updated.
        """
        return Response(
            {'error': 'API keys cannot be updated. Create a new key or delete the existing one.'},
            status=status.HTTP_405_METHOD_NOT_ALLOWED
        )

    def partial_update(self, request, *args, **kwargs):
        """
        API keys cannot be updated.
        """
        return Response(
            {'error': 'API keys cannot be updated. Create a new key or delete the existing one.'},
            status=status.HTTP_405_METHOD_NOT_ALLOWED
        )

    @action(detail=True, methods=['post'])
    def revoke(self, request, pk=None):
        """
        Revoke an API key by setting is_active to False.
        """
        api_key = self.get_object()
        api_key.is_active = False
        api_key.save()
        serializer = self.get_serializer(api_key)
        return Response(serializer.data)

    @action(detail=True, methods=['post'])
    def activate(self, request, pk=None):
        """
        Activate an API key by setting is_active to True.
        """
        api_key = self.get_object()
        if api_key.is_expired():
            return Response(
                {'error': 'Cannot activate an expired API key.'},
                status=status.HTTP_400_BAD_REQUEST
            )
        api_key.is_active = True
        api_key.save()
        serializer = self.get_serializer(api_key)
        return Response(serializer.data)

