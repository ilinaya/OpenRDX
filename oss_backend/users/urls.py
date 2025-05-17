from django.urls import path, include
from rest_framework.routers import DefaultRouter
from .views import UserViewSet, UserGroupViewSet

app_name = 'users'

router = DefaultRouter()
router.register('users', UserViewSet)
router.register(r'groups', UserGroupViewSet)


urlpatterns = [
    path('', include(router.urls)),
]