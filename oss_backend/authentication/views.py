from rest_framework import status
from rest_framework.response import Response
from rest_framework.views import APIView
from rest_framework import permissions
from rest_framework_simplejwt.tokens import RefreshToken
from django.contrib.auth import authenticate
from .serializers import TokenObtainSerializer
from drf_yasg.utils import swagger_auto_schema


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
        serializer = self.serializer_class(data=request.data)
        serializer.is_valid(raise_exception=True)
        
        email = serializer.validated_data.get('email')
        password = serializer.validated_data.get('password')
        
        user = authenticate(request, username=email, password=password)
        
        if user is not None:
            refresh = RefreshToken.for_user(user)
            return Response({
                'refresh': str(refresh),
                'access': str(refresh.access_token),
            })
        
        return Response(
            {"detail": "Invalid credentials"},
            status=status.HTTP_401_UNAUTHORIZED
        )