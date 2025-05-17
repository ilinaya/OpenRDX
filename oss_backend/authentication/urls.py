from django.urls import path
from rest_framework_simplejwt.views import TokenRefreshView
from .views import TokenObtainView

app_name = 'authentication'

urlpatterns = [
    path('token/', TokenObtainView.as_view(), name='token_obtain'),
    path('token/refresh/', TokenRefreshView.as_view(), name='token_refresh'),
]