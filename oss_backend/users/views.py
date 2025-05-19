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