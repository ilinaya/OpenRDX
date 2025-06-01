from rest_framework import viewsets, status
from rest_framework.decorators import action
from rest_framework.response import Response
from django_filters.rest_framework import DjangoFilterBackend
from rest_framework.filters import SearchFilter, OrderingFilter
import json
import redis
import os

from .models import AuthAttributeGroup, RadiusAttribute, UserNasRelationship, Secret
from .serializers import (
    AuthAttributeGroupSerializer, AuthAttributeGroupCreateSerializer,
    RadiusAttributeSerializer, RadiusAttributeCreateSerializer,
    UserNasRelationshipSerializer, UserNasRelationshipCreateSerializer,
    UserNasRelationshipUpdateSerializer,
    SecretSerializer, SecretCreateSerializer, SecretUpdateSerializer
)

class AuthAttributeGroupViewSet(viewsets.ModelViewSet):
    """
    ViewSet for managing authentication attribute groups.
    """
    queryset = AuthAttributeGroup.objects.all()
    serializer_class = AuthAttributeGroupSerializer
    filter_backends = [DjangoFilterBackend, SearchFilter, OrderingFilter]
    filterset_fields = ['name', 'is_system']
    search_fields = ['name', 'description']
    ordering_fields = ['name', 'created_at']
    ordering = ['name']

    @action(detail=False, methods=['get'])
    def list_all(self, request):
        """
        Return a list of all attribute groups without pagination.
        """
        queryset = self.filter_queryset(self.get_queryset())
        serializer = self.get_serializer(queryset, many=True)
        return Response(serializer.data)

    def get_serializer_class(self):
        """
        Return the appropriate serializer class based on the action.
        """
        if self.action == 'create':
            return AuthAttributeGroupCreateSerializer
        return self.serializer_class

    def destroy(self, request, *args, **kwargs):
        """
        Prevent deletion of system attribute groups.
        """
        instance = self.get_object()
        if instance.is_system:
            return Response(
                {"error": "System attribute groups cannot be deleted."},
                status=status.HTTP_400_BAD_REQUEST
            )
        return super().destroy(request, *args, **kwargs)


class RadiusAttributeViewSet(viewsets.ModelViewSet):
    """
    ViewSet for managing RADIUS attributes.
    """
    queryset = RadiusAttribute.objects.all()
    serializer_class = RadiusAttributeSerializer
    filter_backends = [DjangoFilterBackend, SearchFilter, OrderingFilter]
    filterset_fields = ['group', 'vendor_id', 'attribute_id', 'attribute_name', 'attribute_type']
    search_fields = ['attribute_name', 'attribute_value']
    ordering_fields = ['vendor_id', 'attribute_id', 'attribute_name', 'created_at']
    ordering = ['vendor_id', 'attribute_id']

    def get_serializer_class(self):
        """
        Return appropriate serializer class based on the action.
        """
        if self.action == 'create':
            return RadiusAttributeCreateSerializer
        return self.serializer_class

    @action(detail=False, methods=['get'])
    def by_group(self, request):
        """
        Filter attributes by group ID.
        """
        group_id = request.query_params.get('group_id')
        if not group_id:
            return Response(
                {"error": "group_id parameter is required"},
                status=status.HTTP_400_BAD_REQUEST
            )

        attributes = RadiusAttribute.objects.filter(group_id=group_id)
        serializer = self.get_serializer(attributes, many=True)
        return Response(serializer.data)


class UserNasRelationshipViewSet(viewsets.ModelViewSet):
    """
    ViewSet for managing user-NAS relationships.
    """
    queryset = UserNasRelationship.objects.all()
    serializer_class = UserNasRelationshipSerializer
    filter_backends = [DjangoFilterBackend, SearchFilter, OrderingFilter]
    filterset_fields = ['user', 'nas', 'attribute_group']
    search_fields = ['user__email', 'nas__name', 'attribute_group__name']
    ordering_fields = ['created_at', 'updated_at']
    ordering = ['-created_at']

    def get_serializer_class(self):
        """
        Return appropriate serializer class based on the action.
        """
        if self.action == 'create':
            return UserNasRelationshipCreateSerializer
        elif self.action in ['update', 'partial_update']:
            return UserNasRelationshipUpdateSerializer
        return self.serializer_class

    @action(detail=False, methods=['get'])
    def by_user(self, request):
        """
        Filter relationships by user ID.
        """
        user_id = request.query_params.get('user_id')
        if not user_id:
            return Response(
                {"error": "user_id parameter is required"},
                status=status.HTTP_400_BAD_REQUEST
            )

        relationships = UserNasRelationship.objects.filter(user_id=user_id)
        serializer = self.get_serializer(relationships, many=True)
        return Response(serializer.data)

    @action(detail=False, methods=['get'])
    def by_nas(self, request):
        """
        Filter relationships by NAS ID.
        """
        nas_id = request.query_params.get('nas_id')
        if not nas_id:
            return Response(
                {"error": "nas_id parameter is required"},
                status=status.HTTP_400_BAD_REQUEST
            )

        relationships = UserNasRelationship.objects.filter(nas_id=nas_id)
        serializer = self.get_serializer(relationships, many=True)
        return Response(serializer.data)

    @action(detail=True, methods=['post'])
    def change_attribute_group(self, request, pk=None):
        """
        Change the attribute group for a user-NAS relationship and publish to Redis.
        """
        relationship = self.get_object()

        # Get the new attribute group ID from the request
        attribute_group_id = request.data.get('attribute_group_id')
        if not attribute_group_id:
            return Response(
                {"error": "attribute_group_id is required"},
                status=status.HTTP_400_BAD_REQUEST
            )

        try:
            attribute_group = AuthAttributeGroup.objects.get(pk=attribute_group_id)
        except AuthAttributeGroup.DoesNotExist:
            return Response(
                {"error": f"AuthAttributeGroup with id {attribute_group_id} does not exist"},
                status=status.HTTP_404_NOT_FOUND
            )

        # Update the relationship
        relationship.attribute_group = attribute_group
        relationship.save()

        # Publish to Redis
        self._publish_to_redis(relationship, 'change_attribute_group')

        serializer = self.get_serializer(relationship)
        return Response(serializer.data)

    @action(detail=True, methods=['post'])
    def reauth(self, request, pk=None):
        """
        Trigger a reauthentication for a user-NAS relationship by publishing to Redis.
        """
        relationship = self.get_object()

        # Publish to Redis
        self._publish_to_redis(relationship, 'reauth')

        return Response({"message": "Reauthentication request sent"})

    def _publish_to_redis(self, relationship, action_type):
        """
        Publish a message to Redis for CoA or reauth.

        Args:
            relationship: The UserNasRelationship instance
            action_type: Type of action ('change_attribute_group' or 'reauth')
        """
        # Get Redis connection details from environment
        redis_host = os.environ.get('REDIS_HOST', 'localhost')
        redis_port = int(os.environ.get('REDIS_PORT', 6379))
        redis_db = int(os.environ.get('REDIS_DB', 0))
        coa_topic = os.environ.get('COA_TOPIC', 'radius_coa')

        try:
            # Connect to Redis
            r = redis.Redis(host=redis_host, port=redis_port, db=redis_db)

            # Prepare the payload
            payload = {
                'action': action_type,
                'user_id': str(relationship.user.id),
                'username': relationship.user.email,  # Using email as username
                'nas_id': str(relationship.nas.id),
                'nas_ip': relationship.nas.ip_address,
                'nas_name': relationship.nas.name,
                'attribute_group_id': str(relationship.attribute_group.id),
                'attribute_group_name': relationship.attribute_group.name,
                'attribute_overrides': relationship.attribute_overrides
            }

            # Publish to Redis
            r.publish(coa_topic, json.dumps(payload))

        except Exception as e:
            # Log the error but don't fail the request
            print(f"Error publishing to Redis: {e}")


class SecretViewSet(viewsets.ModelViewSet):
    """
    ViewSet for managing RADIUS secrets.
    """
    queryset = Secret.objects.all()
    serializer_class = SecretSerializer
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
            return SecretCreateSerializer
        elif self.action in ['update', 'partial_update']:
            return SecretUpdateSerializer
        return self.serializer_class
