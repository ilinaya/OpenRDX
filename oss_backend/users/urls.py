from django.urls import path, include
from rest_framework.routers import DefaultRouter
from .views import (
    UserViewSet, UserGroupViewSet, UserIdentifierTypeViewSet,
    UserIdentifierViewSet
)

app_name = 'users'

router = DefaultRouter()
router.register('users', UserViewSet)
router.register(r'groups', UserGroupViewSet)
router.register(r'identifier-types', UserIdentifierTypeViewSet)

# Nested router for user identifiers
user_identifiers_router = DefaultRouter()
user_identifiers_router.register(r'identifiers', UserIdentifierViewSet, basename='user-identifier')

urlpatterns = [
    path('', include(router.urls)),
    path('users/<int:user_pk>/', include(user_identifiers_router.urls)),
]