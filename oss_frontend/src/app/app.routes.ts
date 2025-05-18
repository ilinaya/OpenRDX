import {Routes} from '@angular/router';
import {AuthGuard} from './core/auth/auth.guard';

export const routes: Routes = [
    {
        path: 'login',
        loadChildren: () => import('./features/login/login.module').then(m => m.LoginModule),
    },
    {
        path: '',
        canActivate: [AuthGuard],
        loadChildren: () => import('./features/dashboard/dashboard.module').then(m => m.DashboardModule),
    },
    {
        path: 'settings',
        canActivate: [AuthGuard],
        loadChildren: () => import('./features/settings/settings.module').then(m => m.SettingsModule),
    },
    {path: '**', redirectTo: ''},
];