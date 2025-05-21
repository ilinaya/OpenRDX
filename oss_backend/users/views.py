from rest_framework import viewsets, permissions
from rest_framework.response import Response
from rest_framework import status
from rest_framework import permissions

from drf_yasg.utils import swagger_auto_schema
from .models import User, UserGroup, UserIdentifierType, UserIdentifier
from .serializers import (
    UserSerializer, UserCreateSerializer, UserUpdateSerializer, UserGroupSerializer,
    UserGroupTreeSerializer, UserIdentifierTypeSerializer, UserIdentifierSerializer
)
from rest_framework.decorators import action
from rest_framework import serializers

class UserGroupViewSet(viewsets.ModelViewSet):
    """
    ViewSet for viewing and editing User instances.
    """
    queryset = UserGroup.objects.all()
    permission_classes = [permissions.IsAuthenticated]

    def get_serializer_class(self):
        if self.action == 'create':
            return UserGroupSerializer
        elif self.action in ['update', 'partial_update']:
            return UserGroupSerializer
        return UserGroupSerializer

    @swagger_auto_schema(
        operation_description="List all user groups",
        responses={200: UserGroupSerializer(many=True)}
    )
    def list(self, request, *args, **kwargs):
        return super().list(request, *args, **kwargs)

    @action(detail=False, methods=['get'])
    def list_all(self, request):
        """
        Return a list of all admin users without pagination.
        """
        queryset = self.filter_queryset(self.get_queryset())
        serializer = self.get_serializer(queryset, many=True)
        return Response(serializer.data)

    @swagger_auto_schema(
        operation_description="Create a new user group",
        request_body=UserGroupSerializer,
        responses={201: UserGroupSerializer()}
    )
    def create(self, request, *args, **kwargs):
        serializer = self.get_serializer(data=request.data)
        serializer.is_valid(raise_exception=True)
        self.perform_create(serializer)
        headers = self.get_success_headers(serializer.data)
        return Response(
            UserGroupSerializer(instance=serializer.instance).data,
            status=status.HTTP_201_CREATED,
            headers=headers
        )

    @swagger_auto_schema(
        operation_description="Retrieve a user group by ID",
        responses={200: UserGroupSerializer()}
    )
    def retrieve(self, request, *args, **kwargs):
        return super().retrieve(request, *args, **kwargs)

    @swagger_auto_schema(
        operation_description="Update a user group",
        request_body=UserGroupSerializer,
        responses={200: UserGroupSerializer()}
    )
    def update(self, request, *args, **kwargs):
        partial = kwargs.pop('partial', False)
        instance = self.get_object()
        serializer = self.get_serializer(instance, data=request.data, partial=partial)
        serializer.is_valid(raise_exception=True)
        self.perform_update(serializer)
        return Response(UserGroupSerializer(instance=instance).data)

    @swagger_auto_schema(
        operation_description="Delete a user group",
        responses={204: "No content"}
    )
    def destroy(self, request, *args, **kwargs):
        return super().destroy(request, *args, **kwargs)

    @action(detail=False, methods=['get'])
    def tree(self, request):
        queryset = self.get_queryset()
        # Assuming you have a serializer that can handle tree structure
        serializer = self.get_serializer(queryset, many=True)
        return Response(serializer.data)

class UserIdentifierTypeViewSet(viewsets.ModelViewSet):
    queryset = UserIdentifierType.objects.all()
    serializer_class = UserIdentifierTypeSerializer
    permission_classes = [permissions.IsAuthenticated]

    @action(detail=False, methods=['get'])
    def list_all(self, request):
        """
        Return a list of all admin users without pagination.
        """
        queryset = self.filter_queryset(self.get_queryset())
        serializer = self.get_serializer(queryset, many=True)
        return Response(serializer.data)

class UserIdentifierViewSet(viewsets.ModelViewSet):
    serializer_class = UserIdentifierSerializer
    permission_classes = [permissions.IsAuthenticated]

    def get_queryset(self):
        user_id = self.kwargs.get('user_pk')
        return UserIdentifier.objects.filter(user_id=user_id)

    def perform_create(self, serializer):
        user_id = self.kwargs.get('user_pk')
        serializer.save(user_id=user_id)

class UserViewSet(viewsets.ModelViewSet):
    """
    ViewSet for viewing and editing User instances.
    """
    queryset = User.objects.all()
    permission_classes = [permissions.IsAuthenticated]
    
    def get_serializer_class(self):
        if self.action == 'create':
            return UserCreateSerializer
        elif self.action in ['update', 'partial_update']:
            return UserUpdateSerializer
        return UserSerializer
    
    @swagger_auto_schema(
        operation_description="List all users",
        responses={200: UserSerializer(many=True)}
    )
    def list(self, request, *args, **kwargs):
        return super().list(request, *args, **kwargs)
    
    @swagger_auto_schema(
        operation_description="Create a new user",
        request_body=UserCreateSerializer,
        responses={201: UserSerializer()}
    )
    def create(self, request, *args, **kwargs):
        serializer = self.get_serializer(data=request.data)
        serializer.is_valid(raise_exception=True)
        self.perform_create(serializer)
        headers = self.get_success_headers(serializer.data)
        return Response(
            UserSerializer(instance=serializer.instance).data,
            status=status.HTTP_201_CREATED,
            headers=headers
        )
    
    @swagger_auto_schema(
        operation_description="Retrieve a user by ID",
        responses={200: UserSerializer()}
    )
    def retrieve(self, request, *args, **kwargs):
        return super().retrieve(request, *args, **kwargs)
    
    @swagger_auto_schema(
        operation_description="Update a user",
        request_body=UserUpdateSerializer,
        responses={200: UserSerializer()}
    )
    def update(self, request, *args, **kwargs):
        partial = kwargs.pop('partial', False)
        instance = self.get_object()
        serializer = self.get_serializer(instance, data=request.data, partial=partial)
        serializer.is_valid(raise_exception=True)
        self.perform_update(serializer)
        return Response(UserSerializer(instance=instance).data)
    
    @swagger_auto_schema(
        operation_description="Delete a user",
        responses={204: "No content"}
    )
    def destroy(self, request, *args, **kwargs):
        return super().destroy(request, *args, **kwargs)

    @action(detail=True, methods=['get', 'post'])
    def identifiers(self, request, pk=None):
        user = self.get_object()
        if request.method == 'GET':
            identifiers = UserIdentifier.objects.filter(user=user)
            serializer = UserIdentifierSerializer(identifiers, many=True)
            return Response(serializer.data)
        elif request.method == 'POST':
            serializer = UserIdentifierSerializer(data=request.data)
            if serializer.is_valid():
                serializer.save(user=user)
                return Response(serializer.data, status=status.HTTP_201_CREATED)
            return Response(serializer.errors, status=status.HTTP_400_BAD_REQUEST)

    @action(detail=True, methods=['get', 'put', 'delete'], url_path='identifiers/(?P<identifier_id>[^/.]+)')
    def identifier(self, request, pk=None, identifier_id=None):
        user = self.get_object()
        try:
            identifier = UserIdentifier.objects.get(user=user, id=identifier_id)
        except UserIdentifier.DoesNotExist:
            return Response(status=status.HTTP_404_NOT_FOUND)

        if request.method == 'GET':
            serializer = UserIdentifierSerializer(identifier)
            return Response(serializer.data)
        elif request.method == 'PUT':
            serializer = UserIdentifierSerializer(identifier, data=request.data)
            if serializer.is_valid():
                serializer.save()
                return Response(serializer.data)
            return Response(serializer.errors, status=status.HTTP_400_BAD_REQUEST)
        elif request.method == 'DELETE':
            identifier.delete()
            return Response(status=status.HTTP_204_NO_CONTENT)

    def _handle_identifiers(self, user, identifiers_data):
        """Handle updating user identifiers."""
        if not identifiers_data:
            return

        # Get existing identifiers
        existing_identifiers = {str(identifier.id): identifier for identifier in user.identifiers.all()}
        
        # Create a map of existing identifiers by type and value
        existing_by_type_value = {
            (str(identifier.identifier_type_id), identifier.value): identifier 
            for identifier in user.identifiers.all()
        }
        
        # Process each identifier in the request
        for identifier_data in identifiers_data:
            identifier_id = identifier_data.get('id')
            type_id = str(identifier_data.get('identifier_type_id'))
            value = identifier_data.get('value')
            
            # Try to find existing identifier by ID or by type+value combination
            if identifier_id and str(identifier_id) in existing_identifiers:
                identifier = existing_identifiers[str(identifier_id)]
            elif (type_id, value) in existing_by_type_value:
                identifier = existing_by_type_value[(type_id, value)]
            else:
                identifier = None
            
            if identifier:
                # Update existing identifier
                serializer = UserIdentifierSerializer(identifier, data=identifier_data, partial=True)
                if serializer.is_valid():
                    serializer.save()
                else:
                    raise serializers.ValidationError(serializer.errors)
            else:
                # Create new identifier
                serializer = UserIdentifierSerializer(data=identifier_data)
                if serializer.is_valid():
                    serializer.save(user=user)
                else:
                    raise serializers.ValidationError(serializer.errors)

        # Remove identifiers that are no longer in the request
        current_ids = {str(data.get('id')) for data in identifiers_data if data.get('id')}
        current_type_values = {
            (str(data.get('identifier_type_id')), data.get('value'))
            for data in identifiers_data
        }
        
        for identifier_id, identifier in existing_identifiers.items():
            if (identifier_id not in current_ids and 
                (str(identifier.identifier_type_id), identifier.value) not in current_type_values):
                identifier.delete()

    def perform_update(self, serializer):
        """Handle the update of a user instance."""
        instance = serializer.instance
        data = serializer.validated_data

        # Handle groups update
        if 'groups' in data:
            # Clear existing groups and set new ones
            instance.groups.clear()
            instance.groups.add(*data['groups'])

        # Save the user instance
        instance = serializer.save()

        # Handle identifiers update
        identifiers_data = self.request.data.get('identifiers', [])
        self._handle_identifiers(instance, identifiers_data)

        return instance