import {Routes} from '@angular/router';
import {DashboardComponent} from './dashboard.component';

export const routes: Routes = [
  {
    path: '',
    component: DashboardComponent,
    children: [
      {
        path: 'devices',
        loadChildren: () => import('../devices/devices.module').then(m => m.DevicesModule),
      },
      {
        path: 'users',
        loadChildren: () => import('../users/users.module').then(m => m.UsersModule),
      },
      {
        path: 'settings',
        loadChildren: () => import('../settings/settings.module').then(m => m.SettingsModule),
      },
      {path: '', redirectTo: 'devices', pathMatch: 'full'},
    ],
  },
];
