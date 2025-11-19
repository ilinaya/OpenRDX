import logging
from rest_framework import status
from rest_framework.response import Response
from rest_framework.views import APIView
from rest_framework import permissions
from rest_framework_simplejwt.tokens import RefreshToken
from django.contrib.auth import authenticate
from django.contrib.auth import get_user_model
from .serializers import TokenObtainSerializer
from drf_yasg.utils import swagger_auto_schema

logger = logging.getLogger(__name__)
UserModel = get_user_model()


class TokenObtainView(APIView):
    """
    API view for getting JWT tokens.
    """
    serializer_class = TokenObtainSerializer

    permission_classes = [permissions.AllowAny]

    @swagger_auto_schema(
        request_body=TokenObtainSerializer,
        operation_description="Obtain JWT token by providing valid credentials",
        responses={
            200: "Returns refresh and access tokens",
            401: "Invalid credentials"
        }
    )
    def post(self, request):
        logger.info("=" * 80)
        logger.info("AUTHENTICATION REQUEST RECEIVED")
        logger.info("=" * 80)
        
        serializer = self.serializer_class(data=request.data)
        serializer.is_valid(raise_exception=True)
        
        email = serializer.validated_data.get('email')
        password = serializer.validated_data.get('password')
        
        logger.info(f"Attempting authentication for email: {email}")
        logger.info(f"Password provided: {'Yes' if password else 'No'} (length: {len(password) if password else 0})")
        
        # Check if user exists
        try:
            user_by_email = UserModel.objects.get(email=email)
            logger.info(f"User found by email: {user_by_email.id}, username: {user_by_email.username}, email: {user_by_email.email}")
            logger.info(f"User is_active: {user_by_email.is_active}, is_staff: {user_by_email.is_staff}")
            logger.info(f"User password hash (first 50 chars): {user_by_email.password[:50] if user_by_email.password else 'None'}...")
        except UserModel.DoesNotExist:
            logger.warning(f"No user found with email: {email}")
            # Check if user exists by username
            try:
                user_by_username = UserModel.objects.get(username=email)
                logger.warning(f"But user found with username: {email}, id: {user_by_username.id}, email: {user_by_username.email}")
            except UserModel.DoesNotExist:
                logger.error(f"No user found with email or username: {email}")
        
        logger.info("Calling authenticate()...")
        user = authenticate(request, username=email, password=password)
        
        if user is not None:
            logger.info(f"Authentication SUCCESSFUL for user: {user.id} ({user.email})")
            refresh = RefreshToken.for_user(user)
            logger.info("JWT tokens generated successfully")
            logger.info("=" * 80)
            return Response({
                'refresh': str(refresh),
                'access': str(refresh.access_token),
            })
        
        logger.error("Authentication FAILED")
        logger.error(f"authenticate() returned None for email: {email}")
        
        # Try to manually check password
        try:
            user = UserModel.objects.get(email=email)
            logger.info(f"Attempting manual password check...")
            password_check = user.check_password(password)
            logger.info(f"Manual check_password result: {password_check}")
            logger.info(f"User can authenticate: {user.is_active}")
        except UserModel.DoesNotExist:
            logger.error("User does not exist in database")
        except Exception as e:
            logger.error(f"Exception during manual password check: {str(e)}", exc_info=True)
        
        logger.info("=" * 80)
        return Response(
            {"detail": "Invalid credentials"},
            status=status.HTTP_401_UNAUTHORIZED
        )