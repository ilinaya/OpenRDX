import { Component, OnInit, HostListener } from '@angular/core';
import {Router, RouterLink} from '@angular/router';
import { AuthService } from '../../../../core/auth/auth.service';
import { AdminUser } from '../../../../shared/models/admin.model';

@Component({
  selector: 'app-navbar',
  templateUrl: './navbar.component.html',
  imports: [
    RouterLink,
  ],
  styleUrls: ['./navbar.component.scss'],
})
export class NavbarComponent implements OnInit {
  navItems = [
    { name: 'Devices', icon: 'devices', route: '/devices' },
    { name: 'Users', icon: 'devices', route: '/users' },
    { name: 'Settings', icon: 'settings', route: '/settings' },
  ];

  currentUser: AdminUser | null = null;

  constructor(private router: Router, private authService: AuthService) {}

  ngOnInit() {
    this.currentUser = this.authService.getCurrentUser();
  }

  isActive(route: string): boolean {
    return this.router.url.includes(route);
  }


}
