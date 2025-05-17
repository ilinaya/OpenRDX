from django.contrib.auth.hashers import check_password
from rest_framework import viewsets, permissions, status
from rest_framework.response import Response
from rest_framework.decorators import action, api_view, permission_classes
from rest_framework.permissions import AllowAny
from django.contrib.auth.password_validation import validate_password
from django.core.exceptions import ValidationError
from .models import AdminUser, AdminGroup
from .serializers import (
    AdminUserSerializer, AdminUserCreateSerializer,
    AdminGroupSerializer, AdminGroupCreateSerializer
)
from .filters import AdminUserFilter
from .utils import send_invitation_email, send_password_reset_email


class AdminUserViewSet(viewsets.ModelViewSet):
    """
    ViewSet for viewing and editing AdminUser instances.
    """
    queryset = AdminUser.objects.all()
    serializer_class = AdminUserSerializer
    permission_classes = [permissions.IsAuthenticated]
    filterset_class = AdminUserFilter

    @action(detail=False, methods=['get'])
    def list_all(self, request):
        """
        Return a list of all admin users without pagination.
        """
        queryset = self.filter_queryset(self.get_queryset())
        serializer = self.get_serializer(queryset, many=True)
        return Response(serializer.data)

    def get_serializer_class(self):
        """
        Return the appropriate serializer class based on the action.
        """
        if self.action == 'create':
            return AdminUserCreateSerializer
        return self.serializer_class

    def create(self, request, *args, **kwargs):
        """
        Create a new admin user.
        """
        serializer = self.get_serializer(data=request.data)
        serializer.is_valid(raise_exception=True)
        user = self.perform_create(serializer)

        # Send invitation email if email is provided
        if user.email:
            send_invitation_email(user, request)

        headers = self.get_success_headers(serializer.data)
        return Response(serializer.data, status=status.HTTP_201_CREATED, headers=headers)

    def perform_create(self, serializer):
        """
        Save the user and return the instance.
        """
        return serializer.save()

    def update(self, request, *args, **kwargs):
        """
        Update an existing admin user.
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

    @action(detail=True, methods=['post'])
    def activate(self, request, pk=None):
        """
        Activate an admin user.
        """
        user = self.get_object()
        user.is_active = True
        user.save()
        serializer = self.get_serializer(user)
        return Response(serializer.data)

    @action(detail=True, methods=['post'])
    def deactivate(self, request, pk=None):
        """
        Deactivate an admin user.
        """
        user = self.get_object()
        user.is_active = False
        user.save()
        serializer = self.get_serializer(user)
        return Response(serializer.data)

    @action(detail=True, methods=['post'])
    def send_invitation(self, request, pk=None):
        """
        Send an invitation email to the user.
        """
        user = self.get_object()
        if not user.email:
            return Response(
                {"error": "User does not have an email address"},
                status=status.HTTP_400_BAD_REQUEST
            )

        success = send_invitation_email(user, request)
        if success:
            return Response({"message": "Invitation email sent successfully"})
        else:
            return Response(
                {"error": "Failed to send invitation email"},
                status=status.HTTP_500_INTERNAL_SERVER_ERROR
            )

    @action(detail=True, methods=['post'])
    def send_password_reset(self, request, pk=None):
        """
        Send a password reset email to the user.
        """
        user = self.get_object()
        if not user.email:
            return Response(
                {"error": "User does not have an email address"},
                status=status.HTTP_400_BAD_REQUEST
            )

        success = send_password_reset_email(user, request)
        if success:
            return Response({"message": "Password reset email sent successfully"})
        else:
            return Response(
                {"error": "Failed to send password reset email"},
                status=status.HTTP_500_INTERNAL_SERVER_ERROR
            )

    @action(detail=False, methods=['post'])
    def change_password(self, request):
        """
        Change the password for the authenticated admin user.

        Requires old_password and new_password in the request data.
        """
        old_password = request.data.get('old_password')
        new_password = request.data.get('new_password')

        if not old_password or not new_password:
            return Response(
                {'message': 'Both old and new passwords are required'},
                status=status.HTTP_400_BAD_REQUEST
            )

        if not check_password(old_password, request.user.password):
            return Response(
                {'message': 'Current password is incorrect'},
                status=status.HTTP_400_BAD_REQUEST
            )

        # Validate the password
        try:
            validate_password(new_password, request.user)
        except ValidationError as e:
            return Response(
                {'message': str(e)},
                status=status.HTTP_400_BAD_REQUEST
            )

        request.user.set_password(new_password)
        request.user.save()

        return Response(
            {'message': 'Password changed successfully'},
            status=status.HTTP_200_OK
        )


class AdminGroupViewSet(viewsets.ModelViewSet):
    """
    ViewSet for viewing and editing AdminGroup instances.
    """
    queryset = AdminGroup.objects.all()
    serializer_class = AdminGroupSerializer
    permission_classes = [permissions.IsAuthenticated]

    @action(detail=False, methods=['get'])
    def list_all(self, request):
        """
        Return a list of all admin groups without pagination.
        """
        queryset = self.filter_queryset(self.get_queryset())
        serializer = self.get_serializer(queryset, many=True)
        return Response(serializer.data)

    def get_serializer_class(self):
        """
        Return the appropriate serializer class based on the action.
        """
        if self.action == 'create':
            return AdminGroupCreateSerializer
        return self.serializer_class


@api_view(['POST'])
@permission_classes([AllowAny])
def set_password(request):
    """
    Set a password for a user using an invitation token.
    """
    token = request.data.get('token')
    password = request.data.get('password')

    if not token or not password:
        return Response(
            {"error": "Token and password are required"},
            status=status.HTTP_400_BAD_REQUEST
        )

    try:
        user = AdminUser.objects.get(invitation_token=token)

        if not user.is_invitation_valid():
            return Response(
                {"error": "Invitation token has expired"},
                status=status.HTTP_400_BAD_REQUEST
            )

        # Validate the password
        try:
            validate_password(password, user)
        except ValidationError as e:
            return Response(
                {"error": str(e)},
                status=status.HTTP_400_BAD_REQUEST
            )

        # Set the password and clear the invitation token
        user.set_password(password)
        user.invitation_token = None
        user.invitation_expires = None
        user.save()

        return Response({"message": "Password set successfully"})

    except AdminUser.DoesNotExist:
        return Response(
            {"error": "Invalid token"},
            status=status.HTTP_400_BAD_REQUEST
        )


@api_view(['POST'])
@permission_classes([AllowAny])
def reset_password(request):
    """
    Reset password for a user using a reset token.
    """
    token = request.data.get('token')
    password = request.data.get('password')

    if not token or not password:
        return Response(
            {"error": "Token and password are required"},
            status=status.HTTP_400_BAD_REQUEST
        )

    try:
        user = AdminUser.objects.get(reset_token=token)

        if not user.is_reset_token_valid():
            return Response(
                {"error": "Reset token has expired"},
                status=status.HTTP_400_BAD_REQUEST
            )

        # Validate the password
        try:
            validate_password(password, user)
        except ValidationError as e:
            return Response(
                {"error": str(e)},
                status=status.HTTP_400_BAD_REQUEST
            )

        # Set the password and clear the reset token
        user.set_password(password)
        user.reset_token = None
        user.reset_expires = None
        user.save()

        return Response({"message": "Password reset successfully"})

    except AdminUser.DoesNotExist:
        return Response(
            {"error": "Invalid token"},
            status=status.HTTP_400_BAD_REQUEST
        )
