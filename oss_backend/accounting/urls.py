from django.urls import path
from .views import sessions_by_nas, sessions_by_user

app_name = 'accounting'

urlpatterns = [
    path('sessions/nas/', sessions_by_nas, name='sessions-by-nas'),
    path('sessions/user/', sessions_by_user, name='sessions-by-user'),
]