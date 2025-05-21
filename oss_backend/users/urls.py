from django.urls import path, include
from rest_framework_nested import routers
from .views import (
    UserViewSet, UserGroupViewSet, UserIdentifierTypeViewSet,
    UserIdentifierViewSet, UserIdentifierNasAuthorizationViewSet, AvailableNasDevicesView
)

app_name = 'users'

router = routers.DefaultRouter()
router.register(r'users', UserViewSet)
router.register(r'groups', UserGroupViewSet)
router.register(r'identifier-types', UserIdentifierTypeViewSet)

# Nested router for user identifiers
users_router = routers.NestedDefaultRouter(router, r'users', lookup='user')
users_router.register(r'identifiers', UserIdentifierViewSet, basename='user-identifiers')

# Nested router for identifier NAS authorizations
identifiers_router = routers.NestedDefaultRouter(users_router, r'identifiers', lookup='identifier')
identifiers_router.register(r'nas-authorizations', UserIdentifierNasAuthorizationViewSet, basename='identifier-nas-authorizations')

urlpatterns = [
    path('', include(router.urls)),
    path('', include(users_router.urls)),
    path('', include(identifiers_router.urls)),
    path('users/<int:user_pk>/identifiers/<int:identifier_pk>/available-nas/', AvailableNasDevicesView.as_view(), name='available-nas-devices'),
]